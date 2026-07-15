//! `MountedComponent`: one live component instance the runtime renders through.

use core::fmt;

use obscur_diagnostics::{Diagnostic, DiagnosticCode};
use obscur_identity::SignalId;
use obscur_view::boundary::{ComponentFailure, ErrorBoundary, ErrorBoundaryPolicy};
use obscur_view::component::{
    ComponentContext, ComponentInstance, ErasedComponent, InstanceId, PropsError,
};
use obscur_view::lifecycle::{MountContext, UnmountContext};
use obscur_view::state::{SignalGenerations, StateGeneration};
use obscur_view::view::{View, ViewProperty};

use crate::component::GuardedRender;
use crate::recovery::ComponentRecovery;

/// The code every component-render failure is reported under (spec §1.6: codes
/// are stable and not reused for unrelated conditions).
///
/// One code covers every failure a render can report today, because there is
/// exactly one: a node's authored properties not decoding into the component's
/// typed props. A render itself cannot fail — see [`ErasedComponent`].
pub const COMPONENT_PROPS_FAILED: DiagnosticCode = DiagnosticCode::new("view.component.props");

/// One mounted component instance: its bookkeeping record plus the erased value
/// the runtime actually renders (spec §2.1).
///
/// The two halves come from the same factory but are deliberately distinct:
/// [`ComponentInstance`] is plain, inspectable data (identity, lifecycle state,
/// subscription scope), while the [`ErasedComponent`] owns the author's
/// [`LocalState`](obscur_view::state::LocalState) fields. Keeping *this* value
/// alive across frames is precisely what preserves a component's local state;
/// dropping it releases that state and the instance's scoped subscriptions,
/// which is the spec's "automatic cleanup".
///
/// # `needs_update`
///
/// [`ComponentContext::request_update`] writes through a `&mut bool` the caller
/// owns — this struct is that owner. The flag is the component's way of saying
/// "render me again"; the runtime reads it after each render and clears it when
/// it acts, so a request is never serviced twice.
///
/// # The error boundary
///
/// Every component is an error boundary by default (spec §1.7), so every
/// mounted instance owns one — built from the author's
/// [`error_boundary`](obscur_view::component::Component::error_boundary) policy
/// and [`fallback`](obscur_view::component::Component::fallback) when it mounts.
/// [`render`](Self::render) goes *through* it: a failed render is contained,
/// retried or propagated here, never returned raw to the caller.
pub struct MountedComponent {
    /// Identity, lifecycle phase and scoped subscriptions.
    instance: ComponentInstance,
    /// The author's component value, behind the erasure. Holds its local state.
    component: Box<dyn ErasedComponent>,
    /// Set by [`ComponentContext::request_update`] during a render or hook.
    needs_update: bool,
    /// The runtime's dirty mark: this component's output is stale and must be
    /// re-rendered. True from mount (nothing has rendered it yet).
    needs_render: bool,
    /// This component's §1.7 error boundary, governed by the author's policy.
    boundary: ErrorBoundary,
    /// How many failures this boundary has recorded since the component last
    /// rendered cleanly — the attempt the next failure counts as, and so what
    /// spends the policy's retry budget. Reset by a healthy render, which is
    /// what makes the budget per *episode* rather than per instance.
    attempts: u32,
}

impl MountedComponent {
    /// Pairs a freshly minted `instance` record with its `component` value, and
    /// builds the component's error boundary from its own policy and fallback.
    #[must_use]
    pub fn new(instance: ComponentInstance, component: Box<dyn ErasedComponent>) -> Self {
        let mut boundary = ErrorBoundary::new(component.error_boundary_erased());
        if let Some(fallback) = component.fallback_erased() {
            boundary.set_fallback(fallback);
        }
        Self {
            instance,
            component,
            needs_update: false,
            // A fresh instance has produced no output yet.
            needs_render: true,
            boundary,
            attempts: 0,
        }
    }

    /// The §1.7 policy governing this component's boundary.
    #[must_use]
    pub fn boundary_policy(&self) -> ErrorBoundaryPolicy {
        self.boundary.policy()
    }

    /// This component's error boundary.
    #[must_use]
    pub const fn boundary(&self) -> &ErrorBoundary {
        &self.boundary
    }

    /// This instance's identity.
    #[must_use]
    pub const fn instance_id(&self) -> InstanceId {
        self.instance.id()
    }

    /// The bookkeeping record.
    #[must_use]
    pub const fn instance(&self) -> &ComponentInstance {
        &self.instance
    }

    /// Whether the component asked to be re-rendered since the flag was last
    /// cleared.
    #[must_use]
    pub const fn needs_update(&self) -> bool {
        self.needs_update
    }

    /// Whether this component's output is stale and must be re-rendered.
    #[must_use]
    pub const fn needs_render(&self) -> bool {
        self.needs_render
    }

    /// Marks the component's output stale — e.g. because the node's authored
    /// props changed, so its typed props may decode differently.
    pub const fn mark_dirty(&mut self) {
        self.needs_render = true;
    }

    /// Advances the instance to `Mounted` and runs the component's `on_mount`.
    pub fn mount(&mut self, signals: &SignalGenerations) {
        self.instance.mount();
        let Self {
            instance,
            component,
            needs_update,
            ..
        } = self;
        let phase = instance.state();
        let mut ctx = ComponentContext::new(instance.scope_mut(), phase, needs_update, signals);
        let mut mount = MountContext::new(&mut ctx);
        component.on_mount_erased(&mut mount);
    }

    /// Runs the component's `on_unmount`, then retires the instance (clearing its
    /// subscription scope).
    ///
    /// Called *before* the value is dropped, so a component can act while its
    /// state is still readable — releasing the state itself needs no hook.
    pub fn unmount(&mut self, signals: &SignalGenerations) {
        let Self {
            instance,
            component,
            needs_update,
            ..
        } = self;
        let phase = instance.state();
        let mut ctx = ComponentContext::new(instance.scope_mut(), phase, needs_update, signals);
        let mut unmount = UnmountContext::new(&mut ctx);
        component.on_unmount_erased(&mut unmount);
        self.instance.unmount();
    }

    /// Whether this component depends on `source` and has not seen its `current`
    /// generation — i.e. it is a consumer of a value that changed (spec §3.1).
    #[must_use]
    pub fn depends_on_stale(&self, source: SignalId, current: StateGeneration) -> bool {
        self.instance.scope().is_stale_for(source, current)
    }

    /// Renders the component against the node's authored `props`, through its
    /// error boundary (spec §1.7).
    ///
    /// `signals` supplies the current generation of every observable source, so a
    /// dependency recorded by [`ComponentContext::track`] captures the generation
    /// it was actually taken at.
    ///
    /// # A failed render never returns raw
    ///
    /// There is no `Result` here on purpose: a caller handed an error would have
    /// to decide what happens to the subtree, and that decision is the
    /// boundary's. A failure comes back as [`GuardedRender::Failed`] with the
    /// boundary's [`RecoveryResult`](crate::recovery::RecoveryResult) already taken — and with the dirty mark
    /// already set to match it, because only a *retry* wants this component
    /// rendered again with the same props. Leaving it stale after any other
    /// decision would re-fire the failure, and its diagnostic, every frame.
    pub fn render(&mut self, props: &[ViewProperty], signals: &SignalGenerations) -> GuardedRender {
        match self.try_render(props, signals) {
            Ok(view) => {
                // A healthy render ends the failure episode: the next failure
                // starts with a full retry budget rather than inheriting the
                // spent one.
                self.boundary.reset();
                self.attempts = 0;
                GuardedRender::Rendered(view)
            }
            Err(error) => self.contain(&error),
        }
    }

    /// Applies the §1.7 boundary to a render that failed with `error`.
    fn contain(&mut self, error: &PropsError) -> GuardedRender {
        let failure = ComponentFailure::new(
            self.instance.state(),
            self.attempts,
            COMPONENT_PROPS_FAILED,
            error.to_string(),
        );
        self.attempts += 1;
        let result = ComponentRecovery::new().recover(&mut self.boundary, failure.clone());
        // A retry is the one decision that wants this component rendered again
        // as it stands: the props are authored *into* the node, so a later frame
        // can supply ones that decode. Every other decision is final for these
        // props.
        if !result.is_retried() {
            self.needs_render = false;
            self.needs_update = false;
        }
        let fallback = result.is_fallback().then(|| self.fallback());
        GuardedRender::Failed {
            failure,
            result,
            fallback,
        }
    }

    /// Contains `diagnostic` — a failure that escaped a descendant's disabled
    /// boundary — at this boundary, returning the failure as recorded here and
    /// the fallback UI that replaces *this* component's subtree.
    ///
    /// The retry budget belongs to the boundary the failure happened in, so a
    /// boundary catching an escaped failure replaces its subtree rather than
    /// retrying: the replacement unmounts the descendant that failed, which is
    /// what containing it means.
    pub fn contain_escaped(&mut self, diagnostic: Diagnostic) -> (ComponentFailure, View) {
        let failure =
            ComponentFailure::from_diagnostic(self.instance.state(), self.attempts, diagnostic);
        self.attempts += 1;
        self.boundary.record(failure.clone());
        (failure, self.fallback())
    }

    /// The fallback UI to put in place of this component's subtree.
    ///
    /// A component that supplied none falls back to *nothing*: the failed
    /// subtree is emptied rather than left showing the last good output.
    fn fallback(&self) -> View {
        self.boundary.fallback_view().cloned().unwrap_or_default()
    }

    /// Renders the component, reporting a props-decode failure raw.
    ///
    /// # Errors
    ///
    /// Returns the [`PropsError`] when `props` do not satisfy the component's
    /// typed props; the component is not called in that case.
    fn try_render(
        &mut self,
        props: &[ViewProperty],
        signals: &SignalGenerations,
    ) -> Result<View, PropsError> {
        let Self {
            instance,
            component,
            needs_update,
            ..
        } = self;
        // Read the phase before borrowing the scope mutably.
        let phase = instance.state();
        // Dependencies are re-collected every render: a component that reads a
        // source only on some branch must depend on it only when it actually did,
        // so last render's tracking must not linger.
        instance.scope_mut().clear();
        let mut ctx = ComponentContext::new(instance.scope_mut(), phase, needs_update, signals);
        let view = component.render_erased(&mut ctx, props)?;

        // A request raised DURING this render is honoured by the next frame, not
        // by looping here: re-rendering it now would let a component that always
        // requests an update spin the frame forever. So the dirty mark survives
        // exactly when the component asked for it again.
        self.needs_render = self.needs_update;
        self.needs_update = false;
        Ok(view)
    }
}

impl fmt::Debug for MountedComponent {
    /// Shows the record and the update flag; the erased component is opaque by
    /// construction (it has no `Debug` bound — requiring one would force every
    /// author's component to derive it).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MountedComponent")
            .field("instance", &self.instance)
            .field("needs_update", &self.needs_update)
            .field("needs_render", &self.needs_render)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::{COMPONENT_PROPS_FAILED, MountedComponent};
    use crate::component::GuardedRender;
    use crate::recovery::RecoveryResult;
    use obscur_diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticSubsystem};
    use obscur_identity::{ComponentId, InternedSymbol};
    use obscur_view::boundary::{ComponentFailure, ErrorBoundaryPolicy};
    use obscur_view::component::{
        Component, ComponentContext, ComponentInstance, ErasedComponent, FromProps, InstanceId,
        PropsError, decode_required,
    };
    use obscur_view::lifecycle::{LifecycleState, MountContext, UnmountContext};
    use obscur_view::state::SignalGenerations;
    use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn step() -> InternedSymbol {
        InternedSymbol::from_raw(0x8000_0001)
    }

    struct CounterProps {
        step: u32,
    }

    impl FromProps for CounterProps {
        fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
            Ok(Self {
                step: decode_required(props, step())?,
            })
        }
    }

    /// Records its hook calls into a shared log so the ordering is observable.
    struct Counter {
        count: obscur_view::state::LocalState<u32>,
        log: Rc<RefCell<Vec<&'static str>>>,
        ask_for_update: bool,
    }

    impl Component for Counter {
        type Props = CounterProps;

        fn render(&mut self, ctx: &mut ComponentContext<'_>, props: &Self::Props) -> View {
            self.count.set(*self.count.get() + props.step);
            self.log.borrow_mut().push("render");
            if self.ask_for_update {
                ctx.request_update();
            }
            View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(
                self.count.get().to_string(),
            )]))
        }

        fn on_mount(&mut self, _ctx: &mut MountContext<'_, '_>) {
            self.log.borrow_mut().push("mount");
        }

        fn on_unmount(&mut self, _ctx: &mut UnmountContext<'_, '_>) {
            self.log.borrow_mut().push("unmount");
        }
    }

    fn mounted(log: &Rc<RefCell<Vec<&'static str>>>, ask: bool) -> MountedComponent {
        MountedComponent::new(
            ComponentInstance::new(InstanceId::from_raw(1), ComponentId::from_raw(9)),
            Box::new(Counter {
                count: obscur_view::state::LocalState::new(0),
                log: log.clone(),
                ask_for_update: ask,
            }) as Box<dyn ErasedComponent>,
        )
    }

    fn props(by: i64) -> Vec<ViewProperty> {
        vec![ViewProperty::new(step(), ViewValue::Int(by))]
    }

    fn rendered_text(component: &mut MountedComponent, by: i64) -> String {
        let view = component
            .render(&props(by), &SignalGenerations::new())
            .rendered()
            .expect("props decode");
        let ViewNode::Text { content, .. } = &view.root().nodes()[0] else {
            panic!("counter renders text");
        };
        content.to_string()
    }

    #[test]
    fn local_state_persists_because_the_erased_value_is_kept_alive() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        assert_eq!(rendered_text(&mut component, 2), "2");
        assert_eq!(rendered_text(&mut component, 3), "5");
    }

    #[test]
    fn mount_advances_the_lifecycle_and_runs_the_hook() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        assert_eq!(component.instance().state(), LifecycleState::Created);
        component.mount(&SignalGenerations::new());
        assert_eq!(component.instance().state(), LifecycleState::Mounted);
        assert_eq!(&*log.borrow(), &["mount"]);
    }

    #[test]
    fn unmount_runs_the_hook_before_retiring_the_instance() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        component.mount(&SignalGenerations::new());
        component.unmount(&SignalGenerations::new());
        assert_eq!(&*log.borrow(), &["mount", "unmount"]);
        assert_eq!(component.instance().state(), LifecycleState::Unmounted);
        // "Automatic cleanup": the scope is released with the instance.
        assert!(component.instance().scope().is_empty());
    }

    #[test]
    fn a_fresh_instance_is_stale_until_it_renders() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        assert!(component.needs_render(), "nothing has rendered it yet");
        let _ = rendered_text(&mut component, 1);
        assert!(!component.needs_render(), "its output is now current");
    }

    #[test]
    fn request_update_during_render_keeps_it_stale_for_the_next_frame() {
        // The component asks to re-render from inside its own render. Honouring
        // that immediately would let it spin the frame forever, so the request
        // survives as a dirty mark the NEXT frame services.
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, true);
        let _ = rendered_text(&mut component, 1);
        assert!(component.needs_render(), "the request kept it stale");
        assert!(
            !component.needs_update(),
            "the request itself is consumed, not left to re-fire"
        );
    }

    #[test]
    fn marking_dirty_stales_a_component_whose_props_changed() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        let _ = rendered_text(&mut component, 1);
        assert!(!component.needs_render());
        component.mark_dirty();
        assert!(component.needs_render());
    }

    #[test]
    fn undecodable_props_do_not_render_the_component() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        assert!(matches!(
            component.render(&bad_props(), &SignalGenerations::new()),
            GuardedRender::Failed { .. }
        ));
        assert!(log.borrow().is_empty(), "render must not have been called");
    }

    // ---- §1.7: the boundary answers for a failed render ----------------------

    /// A component whose boundary policy and fallback the test chooses.
    struct Guarded {
        policy: ErrorBoundaryPolicy,
        fallback: Option<View>,
    }

    impl Component for Guarded {
        type Props = CounterProps;

        fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
            text_view("healthy")
        }

        fn error_boundary(&self) -> ErrorBoundaryPolicy {
            self.policy
        }

        fn fallback(&self) -> Option<View> {
            self.fallback.clone()
        }
    }

    fn text_view(content: &str) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text(content)]))
    }

    fn text_of(view: &View) -> String {
        let ViewNode::Text { content, .. } = &view.root().nodes()[0] else {
            panic!("expected a text node");
        };
        content.to_string()
    }

    /// Props of the wrong type: `step` wants a `u32` and gets a string.
    fn bad_props() -> Vec<ViewProperty> {
        vec![ViewProperty::new(step(), ViewValue::Text("2".into()))]
    }

    fn guarded(policy: ErrorBoundaryPolicy, fallback: Option<View>) -> MountedComponent {
        let mut component = MountedComponent::new(
            ComponentInstance::new(InstanceId::from_raw(2), ComponentId::from_raw(9)),
            Box::new(Guarded { policy, fallback }) as Box<dyn ErasedComponent>,
        );
        component.mount(&SignalGenerations::new());
        component
    }

    fn fail(component: &mut MountedComponent) -> (ComponentFailure, RecoveryResult, Option<View>) {
        match component.render(&bad_props(), &SignalGenerations::new()) {
            GuardedRender::Failed {
                failure,
                result,
                fallback,
            } => (failure, result, fallback),
            GuardedRender::Rendered(_) => panic!("bad props must not render"),
        }
    }

    #[test]
    fn the_authors_policy_and_fallback_reach_the_boundary() {
        // The mount reads BOTH `error_boundary()` and `fallback()`; the runtime
        // never asks the component again.
        let policy = ErrorBoundaryPolicy::default().with_max_retries(3);
        let component = guarded(policy, Some(text_view("sorry")));
        assert_eq!(component.boundary_policy(), policy);
        assert_eq!(
            component.boundary().fallback_view().map(text_of),
            Some("sorry".to_string())
        );
    }

    #[test]
    fn a_first_failure_is_retried_and_stays_stale() {
        let mut component = guarded(ErrorBoundaryPolicy::default(), Some(text_view("sorry")));
        let (failure, result, fallback) = fail(&mut component);
        assert_eq!(result, RecoveryResult::Retried);
        assert!(fallback.is_none(), "a retry does not replace the subtree");
        assert_eq!(failure.attempt(), 0);
        assert_eq!(failure.phase(), LifecycleState::Mounted);
        assert_eq!(failure.diagnostic().code, COMPONENT_PROPS_FAILED);
        assert_eq!(failure.diagnostic().severity, DiagnosticSeverity::Error);
        assert_eq!(failure.diagnostic().subsystem, DiagnosticSubsystem::View);
        assert!(
            component.needs_render(),
            "the retry is the dirty mark surviving"
        );
        assert!(component.boundary().is_tripped());
    }

    #[test]
    fn an_exhausted_budget_hands_back_the_authors_fallback() {
        let mut component = guarded(ErrorBoundaryPolicy::default(), Some(text_view("sorry")));
        let _ = fail(&mut component);
        let (failure, result, fallback) = fail(&mut component);
        assert_eq!(result, RecoveryResult::FellBack);
        assert_eq!(failure.attempt(), 1, "the budget is spent per episode");
        assert_eq!(fallback.as_ref().map(text_of), Some("sorry".to_string()));
        assert!(
            !component.needs_render(),
            "the fallback is this component's output now; re-rendering it would \
             fail, and re-report, every frame"
        );
    }

    #[test]
    fn a_component_with_no_fallback_falls_back_to_nothing() {
        // Not "keep what was there": the failed subtree is emptied, so no stale
        // output can pass for current.
        let mut component = guarded(ErrorBoundaryPolicy::default().with_max_retries(0), None);
        let (_, result, fallback) = fail(&mut component);
        assert_eq!(result, RecoveryResult::FellBack);
        assert!(fallback.expect("a fallback is committed").root().is_empty());
    }

    #[test]
    fn a_disabled_boundary_propagates_instead_of_retrying_or_falling_back() {
        let mut component = guarded(ErrorBoundaryPolicy::disabled(), Some(text_view("sorry")));
        let (_, result, fallback) = fail(&mut component);
        // The whole point of `disabled()`: no retry, no fallback of its own.
        assert_eq!(result, RecoveryResult::Propagated);
        assert!(fallback.is_none());
        assert!(!component.needs_render());
        assert!(
            component.boundary().is_tripped(),
            "still recorded, so it is still reported"
        );
    }

    #[test]
    fn a_healthy_render_ends_the_failure_episode() {
        let mut component = guarded(ErrorBoundaryPolicy::default(), None);
        assert_eq!(fail(&mut component).1, RecoveryResult::Retried);

        // The props that decode arrive: the component recovers on its own.
        let view = component
            .render(&props(1), &SignalGenerations::new())
            .rendered()
            .expect("good props render");
        assert_eq!(text_of(&view), "healthy");
        assert!(!component.boundary().is_tripped());

        // And the next episode starts with a full budget rather than the spent
        // one — otherwise one bad frame would make every later one fatal.
        let (failure, result, _) = fail(&mut component);
        assert_eq!(failure.attempt(), 0);
        assert_eq!(result, RecoveryResult::Retried);
    }

    #[test]
    fn an_escaped_failure_is_recorded_here_and_replaces_this_subtree() {
        let mut parent = guarded(ErrorBoundaryPolicy::default(), Some(text_view("outer")));
        let escaped = Diagnostic::new(
            COMPONENT_PROPS_FAILED,
            DiagnosticSeverity::Error,
            DiagnosticSubsystem::View,
            "a child blew up",
        );
        let (failure, fallback) = parent.contain_escaped(escaped);
        // Re-recorded in THIS boundary's phase and attempt: it is the boundary
        // answering now.
        assert_eq!(failure.phase(), LifecycleState::Mounted);
        assert_eq!(failure.attempt(), 0);
        assert_eq!(failure.diagnostic().message, "a child blew up");
        assert!(parent.boundary().is_tripped());
        // No retry: containing it replaces this subtree, which unmounts the
        // descendant that failed.
        assert_eq!(text_of(&fallback), "outer");
    }
}
