//! `MountedComponent`: one live component instance the runtime renders through.

use core::fmt;

use obscur_view::component::{
    ComponentContext, ComponentInstance, ErasedComponent, InstanceId, PropsError,
};
use obscur_view::lifecycle::{MountContext, UnmountContext};
use obscur_view::view::{View, ViewProperty};

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
}

impl MountedComponent {
    /// Pairs a freshly minted `instance` record with its `component` value.
    #[must_use]
    pub fn new(instance: ComponentInstance, component: Box<dyn ErasedComponent>) -> Self {
        Self {
            instance,
            component,
            needs_update: false,
            // A fresh instance has produced no output yet.
            needs_render: true,
        }
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
    pub fn mount(&mut self) {
        self.instance.mount();
        let Self {
            instance,
            component,
            needs_update,
            ..
        } = self;
        let phase = instance.state();
        let mut ctx = ComponentContext::new(instance.scope_mut(), phase, needs_update);
        let mut mount = MountContext::new(&mut ctx);
        component.on_mount_erased(&mut mount);
    }

    /// Runs the component's `on_unmount`, then retires the instance (clearing its
    /// subscription scope).
    ///
    /// Called *before* the value is dropped, so a component can act while its
    /// state is still readable — releasing the state itself needs no hook.
    pub fn unmount(&mut self) {
        let Self {
            instance,
            component,
            needs_update,
            ..
        } = self;
        let phase = instance.state();
        let mut ctx = ComponentContext::new(instance.scope_mut(), phase, needs_update);
        let mut unmount = UnmountContext::new(&mut ctx);
        component.on_unmount_erased(&mut unmount);
        self.instance.unmount();
    }

    /// Renders the component against the node's authored `props`.
    ///
    /// # Errors
    ///
    /// Returns the [`PropsError`] when `props` do not satisfy the component's
    /// typed props; the component is not called in that case.
    pub fn render(&mut self, props: &[ViewProperty]) -> Result<View, PropsError> {
        let Self {
            instance,
            component,
            needs_update,
            ..
        } = self;
        // Read the phase before borrowing the scope mutably.
        let phase = instance.state();
        let mut ctx = ComponentContext::new(instance.scope_mut(), phase, needs_update);
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
    use super::MountedComponent;
    use obscur_identity::{ComponentId, InternedSymbol};
    use obscur_view::component::{
        Component, ComponentContext, ComponentInstance, ErasedComponent, FromProps, InstanceId,
        PropsError, decode_required,
    };
    use obscur_view::lifecycle::{LifecycleState, MountContext, UnmountContext};
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
        let view = component.render(&props(by)).expect("props decode");
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
        component.mount();
        assert_eq!(component.instance().state(), LifecycleState::Mounted);
        assert_eq!(&*log.borrow(), &["mount"]);
    }

    #[test]
    fn unmount_runs_the_hook_before_retiring_the_instance() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut component = mounted(&log, false);
        component.mount();
        component.unmount();
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
        let bad = vec![ViewProperty::new(step(), ViewValue::Text("2".into()))];
        assert!(component.render(&bad).is_err());
        assert!(log.borrow().is_empty(), "render must not have been called");
    }
}
