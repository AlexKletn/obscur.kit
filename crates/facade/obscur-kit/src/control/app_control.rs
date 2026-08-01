#![doc = include_str!("app_control.doc/_module.md")]

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

#[doc = include_str!("app_control.doc/ControlState.md")]
#[derive(Default)]
struct ControlState {
    #[doc = include_str!("app_control.doc/requests.md")]
    requests: Vec<ControlRequest>,
    #[doc = include_str!("app_control.doc/bindings.md")]
    bindings: CommandBindings,
    #[doc = include_str!("app_control.doc/revision.md")]
    revision: SourceVersion,
}

#[doc = include_str!("app_control.doc/AppControl.md")]
#[derive(Clone, Default)]
pub struct AppControl {
    state: Rc<RefCell<ControlState>>,
}

impl AppControl {
    #[doc = include_str!("app_control.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("app_control.doc/on_command_at.md")]
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

    #[doc = include_str!("app_control.doc/add_event_listener.md")]
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

    #[doc = include_str!("app_control.doc/set_stylesheets.md")]
    pub fn set_stylesheets(&self, sheets: Vec<String>) {
        self.push(ControlRequest::Stylesheets(sheets));
    }

    #[doc = include_str!("app_control.doc/set_stylesheet.md")]
    pub fn set_stylesheet(&self, css: impl Into<String>) {
        self.set_stylesheets(vec![css.into()]);
    }

    #[doc = include_str!("app_control.doc/with_runtime.md")]
    pub fn with_runtime(&self, action: impl FnOnce(&mut UiRuntime) + 'static) {
        self.push(ControlRequest::Runtime(Box::new(action)));
    }

    #[doc = include_str!("app_control.doc/request_frame.md")]
    pub fn request_frame(&self) {
        self.push(ControlRequest::Frame);
    }

    #[doc = include_str!("app_control.doc/has_pending.md")]
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.state.borrow().requests.is_empty()
    }

    #[doc = include_str!("app_control.doc/pending_bindings.md")]
    #[must_use]
    pub fn pending_bindings(&self) -> usize {
        self.state.borrow().bindings.pending()
    }

    #[doc = include_str!("app_control.doc/binding_count.md")]
    #[must_use]
    pub fn binding_count(&self) -> usize {
        self.state.borrow().bindings.len()
    }

    #[doc = include_str!("app_control.doc/bound_node.md")]
    #[must_use]
    pub fn bound_node(&self, target: &SelectorTarget) -> Option<NodeId> {
        self.state.borrow().bindings.bound_node(target)
    }

    #[doc = include_str!("app_control.doc/apply.md")]
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

    #[doc = include_str!("app_control.doc/push.md")]
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

#[doc = include_str!("app_control.doc/sources.md")]
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
