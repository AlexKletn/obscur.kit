//! [`AppControl`] — the handle application code holds onto a running runtime.

use std::cell::RefCell;
use std::rc::Rc;

use crate::bootstrap::CssPipeline;
use crate::css::source::{CssSource, SourceId, SourceVersion};
use crate::headless::input::SelectorTarget;
use crate::identity::{InternedSymbol, NodeId};
use crate::interaction::event::ListenerPhase;
use crate::runtime::frame::FrameReason;
use crate::runtime::{IntoCommandHandler, UiRuntime};

use super::{CommandBindings, ControlRequest};

/// Everything the handle owns, behind one shared cell.
#[derive(Default)]
struct ControlState {
    /// Queued instructions, in the order they were asked for.
    requests: Vec<ControlRequest>,
    /// Positional registrations and the nodes they are attached to.
    bindings: CommandBindings,
    /// The revision the *next* stylesheet set is stamped with (spec §4.12).
    revision: SourceVersion,
}

/// A cheap, clonable handle an application keeps on its own running runtime
/// (spec §2.6, §4.12, §2.3).
///
/// # The problem it solves
///
/// `App` never yielded the runtime, and an application therefore
/// could not reach capabilities the runtime implements and the spec requires:
///
/// - §2.6's routing chain is *focused node → focus scope → ancestors → window →
///   application fallback*, and only the last link was reachable, because the first
///   four are keyed by a [`NodeId`] no application could obtain. One flat command
///   namespace was the consequence: the same symbol could not mean different things
///   in different places, and a handler could not sit beside the markup it serves.
/// - §4.12 requires CSS parsing to be "cached per unique source/version and
///   invalidated **explicitly**". `App` collected the authored sheets once before
///   the loop and nothing ever re-supplied them, so every style edit cost a rebuild.
/// - §2.3's capture/bubble listeners are per-node too, so they were unreachable for
///   the same reason as the routing links.
///
/// # The frame-lifecycle contract
///
/// This handle does **not** hand out a `&mut UiRuntime`, and that is deliberate.
/// Application code runs inside the frame — a command handler is called *during*
/// §2.3's propagation, with the runtime's handler table mutably borrowed by the walk
/// that is calling it — so touching the runtime there would be re-entrancy, and
/// retaining it would outlive the borrow that made it available. So every method
/// here **records** what to do, and the host applies it at one defined point:
///
/// > [`apply`](Self::apply) runs once per frame, after the frame has committed and
/// > before the next one is built.
///
/// > [`apply`](Self::apply) runs once per frame, at the frame boundary: before the
/// > next frame is built, and after the previous one has committed.
///
/// Two properties follow, and both matter:
///
/// - the tree exists when names are resolved (a `NodeId` is only real after a
///   commit), which is what makes positional registration possible at all;
/// - a request made from a handler is applied after that handler returns, never
///   inside it, so nothing an application writes can re-enter the frame.
///
/// A queued request is therefore *not* immediate: a stylesheet replaced from a click
/// handler is installed at the start of the next frame and is visible on that frame.
/// That is one frame, not a reload.
///
/// # Registration and clones
///
/// Every clone names one shared queue, so a handler may capture a clone and use it —
/// which is the point. Methods take `&self` for the same reason.
#[derive(Clone, Default)]
pub struct AppControl {
    state: Rc<RefCell<ControlState>>,
}

impl AppControl {
    /// A handle with nothing queued.
    ///
    /// `App::control` hands out clones of the one it
    /// made in `App::new`; construct one directly only when
    /// wiring a runtime by hand (a guest host, or a test).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `handler` to run when `command` reaches the node `target` names —
    /// links 1 and 3 of §2.6's routing chain.
    ///
    /// This is what makes routing **positional**: the same command symbol runs
    /// different logic depending on which node the command originated at. A handler
    /// here claims the command and stops the chain, so no ancestor, no window
    /// handler and no application fallback sees it.
    ///
    /// `target` is a §7.5 stable identifier — `test-id="save"` written on the markup
    /// itself, an `automation-id`, or a [`NodeId`] read back from the tree — because
    /// a node id does not exist before the first frame and an application configures
    /// itself before `run`. The name is resolved after every commit and re-resolved
    /// if its node is replaced, so the registration follows the element.
    ///
    /// `handler` may take nothing or take the propagating
    /// [`UiEvent`](crate::interaction::event::UiEvent), exactly as
    /// [`UiRuntime::on_command_at`] accepts both.
    ///
    /// Registering the same `(target, command)` twice queues two registrations; the
    /// second replaces the first at the runtime, because a per-node *handler* is
    /// singular (a *listener* is not — see
    /// [`add_event_listener`](Self::add_event_listener)).
    pub fn on_command_at<M>(
        &self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        let handler = Rc::new(RefCell::new(handler.into_command_handler()));
        self.state
            .borrow_mut()
            .bindings
            .push_handler(target, command, handler);
    }

    /// Registers a §2.3 **listener** for `command` on the node `target` names,
    /// running in `phase`.
    ///
    /// A listener observes and steers a command as it propagates, where
    /// [`on_command_at`](Self::on_command_at) registers the one handler that claims
    /// it. With [`ListenerPhase::Capture`] it sees a command aimed at a *descendant*
    /// before the target does — which is how an overlay or a gesture arbiter takes
    /// an interaction away — and it may call
    /// [`stop_propagation`](crate::interaction::event::UiEvent::stop_propagation) or
    /// [`prevent_default`](crate::interaction::event::UiEvent::prevent_default) to
    /// steer what happens next.
    ///
    /// Several listeners per node are allowed and run in registration order.
    pub fn add_event_listener<M>(
        &self,
        target: SelectorTarget,
        command: InternedSymbol,
        phase: ListenerPhase,
        listener: impl IntoCommandHandler<M>,
    ) {
        let listener = Rc::new(RefCell::new(listener.into_command_handler()));
        self.state
            .borrow_mut()
            .bindings
            .push_listener(target, command, phase, listener);
    }

    /// Replaces the authored stylesheet set with `sheets`, in order (spec §4.12).
    ///
    /// This is §4.12's explicit invalidation, from application code: a theme switch,
    /// a re-read of a `.css` file on disk, a density change. The new set replaces the
    /// old one wholesale — it is not appended — and the UA default sheet is
    /// untouched, so authored rules keep winning over it.
    ///
    /// Each source is given a stable [`SourceId`] by position and stamped with a new
    /// [`SourceVersion`], which is what the parse cache is keyed by: the cache holds
    /// one entry per id, so a replacement re-parses rather than growing, and a sheet
    /// that keeps its position keeps its identity. Every sheet in the set is
    /// re-parsed, because replacing the set is a statement about the whole set and
    /// this API is handed strings rather than revisions — a host that wants
    /// per-source revisions drives [`CssPipeline`] with its own [`CssSource`]s.
    ///
    /// Applied on the next [`apply`](Self::apply); several calls in one turn collapse
    /// to the last, since only one set can be current. Installing schedules a frame.
    pub fn set_stylesheets(&self, sheets: Vec<String>) {
        self.push(ControlRequest::Stylesheets(sheets));
    }

    /// [`set_stylesheets`](Self::set_stylesheets) for the common case of one sheet
    /// replacing the whole authored set.
    pub fn set_stylesheet(&self, css: impl Into<String>) {
        self.set_stylesheets(vec![css.into()]);
    }

    /// Queues `action` to run once against the runtime at the next
    /// [`apply`](Self::apply).
    ///
    /// The general escape hatch, for the runtime facilities this handle does not
    /// wrap — `scroll_into_view`, `set_content_offset`, `open_focus_scope`,
    /// `on_scope_command`, a `register_component`. It is a queued `FnOnce` rather
    /// than a `&mut UiRuntime` accessor for the reason the type documents: at the
    /// moment application code runs, the runtime is mid-frame. Here it is not, and
    /// the borrow lasts exactly the call — `action` cannot store it.
    ///
    /// Queue it from anywhere, including a command handler. It runs after that
    /// handler has returned, so it can safely do things a handler must not: mutate
    /// the tree's registrations, replace the root view, or dispatch a command.
    pub fn with_runtime(&self, action: impl FnOnce(&mut UiRuntime) + 'static) {
        self.push(ControlRequest::Runtime(Box::new(action)));
    }

    /// Asks for a frame, for a state change the runtime cannot have observed.
    ///
    /// Not needed after a command handler — the runtime schedules a frame for the
    /// input that ran it — but needed when application state changed for a reason the
    /// runtime never saw: a timer, a background task landing, a message from another
    /// thread's channel.
    pub fn request_frame(&self) {
        self.push(ControlRequest::Frame);
    }

    /// Whether a **queued request** is waiting for [`apply`](Self::apply).
    ///
    /// A host polls this to decide whether an otherwise idle loop owes a turn: a
    /// stylesheet replaced from a background callback has to reach the runtime even
    /// though no input arrived.
    ///
    /// Deliberately *not* about unresolved registrations. A name that matches nothing
    /// stays pending forever, so a loop that woke for that would spin at the frame
    /// rate for the rest of the application's life over one misspelled `test-id`. Ask
    /// [`pending_bindings`](Self::pending_bindings) for those.
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.state.borrow().requests.is_empty()
    }

    /// How many positional registrations are still waiting for their name to resolve.
    ///
    /// A name that matches nothing costs one tree walk per committed frame and never
    /// fires; a non-zero count that never falls is the symptom, and
    /// [`bound_node`](Self::bound_node) says which name it is.
    #[must_use]
    pub fn pending_bindings(&self) -> usize {
        self.state.borrow().bindings.pending()
    }

    /// How many positional registrations this handle holds, resolved or not.
    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.state.borrow().bindings.len()
    }

    /// The node `target`'s registrations are currently attached to, or [`None`] while
    /// the name has not resolved.
    ///
    /// Answers "did my `test-id` actually reach an element?" without a tree walk of
    /// the caller's own — the honest check for a binding that silently never fires.
    #[must_use]
    pub fn bound_node(&self, target: &SelectorTarget) -> Option<NodeId> {
        self.state.borrow().bindings.bound_node(target)
    }

    /// Applies everything queued and (re)binds every positional registration,
    /// returning how many actions were taken.
    ///
    /// **Host contract.** Call at the frame boundary — before a frame is built and
    /// after one has committed — never from inside a handler, and always with the same
    /// [`CssPipeline`] the authored CSS was installed through (the pipeline owns
    /// §4.12's parse cache; a fresh one throws it away).
    ///
    /// `App`'s window runner calls it on both edges, and each edge
    /// answers a different need: before the build, so a request queued since the last
    /// frame reaches *this* one; after the commit, so a name resolves against the tree
    /// that now exists. A guest host embedding the runtime does the same in its own
    /// loop. Calling it more often is safe — with nothing queued and every name bound
    /// it is a handful of borrows — but calling it *less* than once per commit delays
    /// a registration by a frame.
    ///
    /// Nothing here calls into application code except a
    /// [`with_runtime`](Self::with_runtime) action, and the queue borrow is released
    /// before any of them runs — so an action (or a diagnostic sink reached by a CSS
    /// install) may queue further work, which is applied on the next call rather than
    /// re-entering this one.
    pub fn apply(&self, runtime: &mut UiRuntime, css: &mut CssPipeline) -> usize {
        // Drain first and release the borrow: an action may queue more work, and a
        // handle held by application code must not find the queue locked.
        let requests = std::mem::take(&mut self.state.borrow_mut().requests);
        let applied = requests.len();
        let mut stylesheets = None;
        for request in requests {
            match request {
                // Only one set can be current, so several replacements in one turn
                // cost one install rather than one each.
                ControlRequest::Stylesheets(sheets) => stylesheets = Some(sheets),
                ControlRequest::Runtime(action) => action(runtime),
                // A style/state change the runtime did not cause is still a UI
                // commit: the next due frame rebuilds and restyles from it.
                ControlRequest::Frame => {
                    runtime.request_frame(FrameReason::UiCommit);
                }
            }
        }
        if let Some(sheets) = stylesheets {
            let revision = {
                let mut state = self.state.borrow_mut();
                state.revision = state.revision.next();
                state.revision
            };
            css.install(runtime, &sources(&sheets, revision));
            runtime.request_frame(FrameReason::UiCommit);
        }
        applied + self.state.borrow_mut().bindings.apply(runtime)
    }

    /// Queues one request.
    fn push(&self, request: ControlRequest) {
        self.state.borrow_mut().requests.push(request);
    }
}

impl std::fmt::Debug for AppControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.borrow();
        f.debug_struct("AppControl")
            .field("queued", &state.requests.len())
            .field("bindings", &state.bindings)
            .field("revision", &state.revision.raw())
            .finish()
    }
}

/// The authored sheets as identified, versioned CSS sources (spec §4.12).
///
/// Position is identity: the first sheet is always `SourceId(1)`, so a sheet that
/// keeps its place keeps its cache entry across a replacement instead of piling up a
/// new one. `SourceId(0)` is reserved for anonymous sources — which are never
/// cached, which is what the authored set used to be.
fn sources(sheets: &[String], revision: SourceVersion) -> Vec<CssSource> {
    sheets
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let position = u32::try_from(index + 1).unwrap_or(u32::MAX);
            CssSource::new(
                SourceId::from_raw(position),
                format!("app-{position}.css"),
                text.as_str(),
            )
            .at_version(revision)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{AppControl, SourceVersion, sources};
    use crate::bootstrap::CssPipeline;
    use crate::css::source::SourceId;
    use crate::geometry::Size;
    use crate::identity::RuntimeId;
    use crate::runtime::UiRuntime;

    fn runtime() -> UiRuntime {
        UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 100.0))
    }

    #[test]
    fn authored_sheets_are_identified_and_versioned() {
        // The §4.12 key: an anonymous source is never cached, so identity is the
        // difference between a cache that works and one that cannot.
        let built = sources(
            &[".a {}".to_owned(), ".b {}".to_owned()],
            SourceVersion::INITIAL.next(),
        );
        assert_eq!(built[0].id(), SourceId::from_raw(1));
        assert_eq!(built[1].id(), SourceId::from_raw(2));
        assert!(
            built
                .iter()
                .all(|source| source.id() != SourceId::ANONYMOUS)
        );
        assert!(
            built
                .iter()
                .all(|source| source.version() == SourceVersion::INITIAL.next())
        );
    }

    #[test]
    fn a_replacement_bumps_the_revision_rather_than_growing_the_cache() {
        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();

        control.set_stylesheet(".box { width: 10px }");
        control.apply(&mut runtime, &mut css);
        assert_eq!(css.cached_sources(), 1, "the sheet is cached at all");

        control.set_stylesheet(".box { width: 20px }");
        control.apply(&mut runtime, &mut css);
        assert_eq!(
            css.cached_sources(),
            1,
            "the same position replaced its entry instead of adding one"
        );
    }

    #[test]
    fn several_replacements_in_one_turn_collapse_to_the_last() {
        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        control.set_stylesheets(vec![".a {}".to_owned(), ".b {}".to_owned()]);
        control.set_stylesheet(".only {}");
        control.apply(&mut runtime, &mut css);
        assert_eq!(css.sheets().len(), 1, "the last set is the current one");
    }

    #[test]
    fn a_queued_action_runs_once_and_only_on_apply() {
        use std::cell::Cell;
        use std::rc::Rc;

        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        let runs = Rc::new(Cell::new(0_u32));
        let counter = Rc::clone(&runs);
        control.with_runtime(move |_runtime| counter.set(counter.get() + 1));

        assert_eq!(runs.get(), 0, "queued, not run");
        assert!(control.has_pending());
        control.apply(&mut runtime, &mut css);
        assert_eq!(runs.get(), 1);
        control.apply(&mut runtime, &mut css);
        assert_eq!(runs.get(), 1, "a request is applied once, not every frame");
        assert!(!control.has_pending());
    }

    #[test]
    fn an_action_may_queue_more_work_without_locking_the_queue() {
        // The re-entrancy property the type promises: the queue borrow is released
        // before any action runs, so an action that queues is not a panic.
        let mut runtime = runtime();
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        let inner = control.clone();
        control.with_runtime(move |_runtime| inner.request_frame());
        control.apply(&mut runtime, &mut css);
        assert!(
            control.has_pending(),
            "the nested request landed on the next turn's queue"
        );
    }

    #[test]
    fn every_clone_names_one_queue() {
        let control = AppControl::new();
        control.clone().request_frame();
        assert!(control.has_pending());
    }
}
