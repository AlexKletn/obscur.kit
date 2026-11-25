//! `SyntheticDispatcher`: replaying a [`SyntheticInput`] through the real input path.

use obscur_identity::NodeId;
use obscur_interaction::pointer::{
    PointerButton as EventButton, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};
use obscur_runtime::UiRuntime;

use crate::input::{DispatchOutcome, PointerButton, SelectorResolver, SyntheticInput};

#[doc = include_str!("../../docs/api/input/synthetic_dispatcher.doc/SyntheticDispatcher.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SyntheticDispatcher {
    resolver: SelectorResolver,
    timestamp_micros: u64,
}

impl SyntheticDispatcher {
    /// Creates a dispatcher.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            resolver: SelectorResolver::new(),
            timestamp_micros: 0,
        }
    }

    pub(crate) const fn at_time(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    #[doc = include_str!("../../docs/api/input/synthetic_dispatcher.doc/dispatch.md")]
    pub fn dispatch(self, runtime: &mut UiRuntime, input: &SyntheticInput) -> DispatchOutcome {
        match input {
            SyntheticInput::PointerMove { target } => {
                self.pointer(runtime, target, PointerEventKind::Move, None)
            }
            SyntheticInput::PointerDown { target, button } => {
                self.pointer(runtime, target, PointerEventKind::Down, Some(*button))
            }
            SyntheticInput::PointerUp { target, button } => {
                self.pointer(runtime, target, PointerEventKind::Up, Some(*button))
            }
            SyntheticInput::Key(event) => {
                let focused = runtime.focused();
                runtime.set_modifiers(event.modifiers());
                let dispatch = runtime.handle_key(*event);
                DispatchOutcome {
                    target: dispatch.target(),
                    command: dispatch.command(),
                    default_prevented: dispatch.default_prevented(),
                    propagation_path: dispatch
                        .target()
                        .map_or_else(Vec::new, |node| ancestors_of(runtime, node)),
                    focus_changed: focused != runtime.focused(),
                    ..DispatchOutcome::default()
                }
            }
            // Legacy numeric codes have no portable keyboard mapping.
            SyntheticInput::KeyDown { .. } | SyntheticInput::KeyUp { .. } => {
                DispatchOutcome::unresolved()
            }
        }
    }

    #[doc = include_str!("../../docs/api/input/synthetic_dispatcher.doc/dispatch_all.md")]
    pub fn dispatch_all(
        self,
        runtime: &mut UiRuntime,
        inputs: &[SyntheticInput],
    ) -> Vec<DispatchOutcome> {
        inputs
            .iter()
            .map(|input| self.dispatch(runtime, input))
            .collect()
    }

    /// Aims a pointer event of `kind` at `target` and sends it through the real
    /// input path. Button transitions update the held-button mask. Moves carry
    /// the current mask, and releasing one button keeps the others held.
    fn pointer(
        self,
        runtime: &mut UiRuntime,
        target: &crate::input::SelectorTarget,
        kind: PointerEventKind,
        button: Option<PointerButton>,
    ) -> DispatchOutcome {
        let Some(point) = self.resolver.point_for(runtime, target) else {
            return DispatchOutcome::unresolved();
        };
        // The headless write-half is a first-party driver standing in for a real
        // user, so it vouches for the gesture: its events are marked TRUSTED, the
        // way a platform adapter marks a real OS event. This is what lets a
        // headless assertion exercise a click's gated default action (§2.3 step 4,
        // enforced in `handle_pointer`) — "command dispatch behaves exactly as it
        // would for a real user". An UNTRUSTED synthetic click (the public
        // programmatic API, or an external DevTools inject) is a different case and
        // has its own coverage; it is deliberately NOT what this harness produces.
        let mut event = PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, point)
            .with_timestamp(self.timestamp_micros)
            .as_trusted();
        let mut buttons = runtime.mouse_buttons();
        if let Some(button) = button {
            let button = match button {
                PointerButton::Primary => EventButton::Primary,
                PointerButton::Secondary => EventButton::Secondary,
                PointerButton::Middle => EventButton::Middle,
            };
            if kind == PointerEventKind::Down {
                buttons.press(button);
            } else {
                buttons.release(button);
            }
            event = event.with_button(button);
        }
        let dispatch = runtime.handle_pointer(event.with_buttons(buttons));

        DispatchOutcome {
            target: dispatch.target,
            propagation_path: dispatch
                .target
                .map(|node| ancestors_of(runtime, node))
                .unwrap_or_default(),
            command: dispatch.command,
            hover_changed: dispatch.hover_changed,
            focus_changed: dispatch.focus_changed,
            default_prevented: dispatch.default_prevented,
        }
    }
}

#[doc = include_str!("../../docs/api/input/synthetic_dispatcher.doc/ancestors_of.md")]
fn ancestors_of(runtime: &UiRuntime, node: NodeId) -> Vec<NodeId> {
    let mut path = vec![node];
    let mut current = node;
    // The tree is finite and acyclic, but bound the walk anyway: a malformed
    // parent link must not hang a test.
    for _ in 0..runtime.tree().len() {
        let Some(parent) = runtime
            .tree()
            .get(current)
            .and_then(|n| n.logical_parent().parent())
        else {
            break;
        };
        if parent == current {
            break;
        }
        path.push(parent);
        current = parent;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::SyntheticDispatcher;
    use crate::input::{PointerButton, SelectorTarget, SyntheticInput};
    use obscur_geometry::{Point, Size};
    use obscur_identity::{ComponentId, InternedSymbol, RuntimeId};
    use obscur_runtime::UiRuntime;
    use obscur_style::cascade::CascadeOrigin;
    use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
    use obscur_style::property::{Length, PropertyId, PropertyValue};
    use obscur_style::selector::{ComplexSelector, CompoundSelector, SelectorList, SimpleSelector};
    use obscur_view::view::{
        CLASS, EVENT_CLICK, EVENT_KEY_DOWN, EVENT_POINTER_UP, TEST_ID, View, ViewEventBinding,
        ViewFragment, ViewNode, ViewProperty, ViewValue,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    const SUBMIT: InternedSymbol = InternedSymbol::from_raw(0x9001);
    const RELEASE: InternedSymbol = InternedSymbol::from_raw(0x9002);
    const KEY_DOWN: InternedSymbol = InternedSymbol::from_raw(0x9003);

    /// One clickable element tagged `test-id="submit"`, bound to SUBMIT.
    ///
    /// It also carries `class="box"` so the sheet below gives it a real size: an
    /// auto-sized element lays out 0x0 and nothing can hit-test it, which would
    /// make these tests pass vacuously.
    fn view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(1))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
                .with_prop(ViewProperty::new(TEST_ID, ViewValue::Text("submit".into())))
                .with_event(ViewEventBinding::new(EVENT_CLICK, SUBMIT))
                .with_event(ViewEventBinding::new(EVENT_KEY_DOWN, KEY_DOWN))
                .with_event(ViewEventBinding::new(EVENT_POINTER_UP, RELEASE)),
        ]))
    }

    /// `.box { width: 100px; height: 50px }`.
    fn sheet() -> Stylesheet {
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                SelectorList::new(vec![ComplexSelector::new(
                    CompoundSelector::new(vec![SimpleSelector::Class("box".into())]),
                    Vec::new(),
                )]),
                vec![
                    BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(100.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(50.0)),
                    ),
                ],
            )],
        )
    }

    /// A runtime showing `view()`, with SUBMIT counting its invocations.
    fn runtime() -> (UiRuntime, Rc<Cell<u32>>) {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
        let clicks = Rc::new(Cell::new(0));
        {
            let clicks = clicks.clone();
            runtime.on_command(SUBMIT, move || clicks.set(clicks.get() + 1));
        }
        runtime.render_now(&view(), &[sheet()]);
        (runtime, clicks)
    }

    #[test]
    fn a_click_by_test_id_runs_the_bound_command() {
        // THE point of §7.4/§7.5: a test names a node by a stable selector and the
        // real input path does the rest — hit-test, press/release pairing, command
        // dispatch. Nothing here reaches past `handle_pointer`.
        let (mut runtime, clicks) = runtime();
        let outcomes = SyntheticDispatcher::new().dispatch_all(
            &mut runtime,
            &[
                SyntheticInput::pointer_down(
                    SelectorTarget::test_id("submit"),
                    PointerButton::Primary,
                ),
                SyntheticInput::pointer_up(
                    SelectorTarget::test_id("submit"),
                    PointerButton::Primary,
                ),
            ],
        );

        assert_eq!(clicks.get(), 1, "the bound command actually ran");
        assert!(outcomes[0].resolved(), "the press landed on the node");
        assert_eq!(
            outcomes[1].command,
            Some(SUBMIT),
            "the release completed the click and reports its default action"
        );
    }

    #[test]
    fn reports_prevented_keyboard_activation() {
        use obscur_interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
        use obscur_interaction::event::UiEvent;

        let (mut runtime, clicks) = runtime();
        let dispatcher = SyntheticDispatcher::new();
        let target = dispatcher
            .dispatch(
                &mut runtime,
                &SyntheticInput::pointer_move(SelectorTarget::test_id("submit")),
            )
            .target
            .unwrap();
        assert!(runtime.focus_node(Some(target)));
        let cancel = Rc::new(Cell::new(true));
        let veto = cancel.clone();
        runtime.on_command(KEY_DOWN, move |event: &mut UiEvent| {
            if veto.get() {
                event.prevent_default();
            }
        });
        let press = SyntheticInput::key(
            KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::Named(NamedKey::Enter)))
                .as_trusted(),
        );
        let stopped = dispatcher.dispatch(&mut runtime, &press);
        assert_eq!(stopped.target, Some(target));
        assert!(stopped.default_prevented);
        assert_eq!(stopped.command, None);
        assert_eq!(clicks.get(), 0);

        cancel.set(false);
        let activated = dispatcher.dispatch(&mut runtime, &press);
        assert!(!activated.default_prevented);
        assert_eq!(activated.command, Some(SUBMIT));
        assert_eq!(clicks.get(), 1);
    }

    #[test]
    fn reports_a_prevented_click_separately_from_an_unbound_action() {
        let (mut runtime, clicks) = runtime();
        runtime.on_command(RELEASE, |event: &mut obscur_interaction::event::UiEvent| {
            event.prevent_default();
        });
        let target = SelectorTarget::test_id("submit");
        let dispatcher = SyntheticDispatcher::new();
        let press = dispatcher.dispatch(
            &mut runtime,
            &SyntheticInput::pointer_down(target.clone(), PointerButton::Primary),
        );
        let release = dispatcher.dispatch(
            &mut runtime,
            &SyntheticInput::pointer_up(target, PointerButton::Primary),
        );
        assert!(press.resolved() && release.resolved());
        assert!(!press.default_prevented);
        assert!(release.default_prevented);
        assert_eq!(release.command, None);
        assert_eq!(clicks.get(), 0);
    }

    #[test]
    fn recording_and_replay_keep_deterministic_pointer_timestamps() {
        use crate::host::HeadlessHost;
        use crate::replay::{HeadlessRecorder, ReplaySession, Sensitivity};
        use obscur_interaction::event::UiEvent;
        use std::cell::RefCell;

        fn host() -> (HeadlessHost, Rc<RefCell<Vec<u64>>>) {
            let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
            let timestamps = Rc::new(RefCell::new(Vec::new()));
            let sink = timestamps.clone();
            host.runtime_mut()
                .on_command(SUBMIT, move |event: &mut UiEvent| {
                    sink.borrow_mut().push(event.pointer().unwrap().timestamp());
                });
            host.render_styled(&view(), &[sheet()]);
            (host, timestamps)
        }

        let (mut original, timestamps) = host();
        original.advance_time(700_500);
        let mut recorder = HeadlessRecorder::new(&mut original);
        let target = SelectorTarget::test_id("submit");
        for delay in [0, 600_000_000] {
            recorder.advance_time(delay);
            recorder.input(
                SyntheticInput::pointer_down(target.clone(), PointerButton::Primary),
                Sensitivity::Normal,
            );
            recorder.input(
                SyntheticInput::pointer_up(target.clone(), PointerButton::Primary),
                Sensitivity::Normal,
            );
        }
        assert_eq!(*timestamps.borrow(), [700, 600_700]);

        let (mut replay, replayed) = host();
        let result =
            ReplaySession::new(recorder.finish()).replay_on_host(&mut replay, &view(), &[sheet()]);
        assert!(result.is_complete());
        assert_eq!(*replayed.borrow(), [700, 600_700]);
    }

    #[test]
    fn secondary_release_preserves_an_outstanding_primary_press() {
        let (mut runtime, clicks) = runtime();
        let target = SelectorTarget::test_id("submit");
        let dispatcher = SyntheticDispatcher::new();

        for button in [PointerButton::Primary, PointerButton::Secondary] {
            dispatcher.dispatch(
                &mut runtime,
                &SyntheticInput::pointer_down(target.clone(), button),
            );
        }
        dispatcher.dispatch(&mut runtime, &SyntheticInput::pointer_move(target.clone()));
        let held = runtime.mouse_buttons();
        assert!(held.contains(super::EventButton::Primary));
        assert!(held.contains(super::EventButton::Secondary));

        let secondary = dispatcher.dispatch(
            &mut runtime,
            &SyntheticInput::pointer_up(target.clone(), PointerButton::Secondary),
        );
        assert_eq!(secondary.command, None);
        assert_eq!(clicks.get(), 0);
        assert!(
            runtime
                .mouse_buttons()
                .contains(super::EventButton::Primary)
        );
        assert!(
            !runtime
                .mouse_buttons()
                .contains(super::EventButton::Secondary)
        );

        let primary = dispatcher.dispatch(
            &mut runtime,
            &SyntheticInput::pointer_up(target, PointerButton::Primary),
        );
        assert_eq!(primary.command, Some(SUBMIT));
        assert_eq!(clicks.get(), 1);
        assert!(runtime.mouse_buttons().is_empty());
    }

    #[test]
    fn a_click_by_coordinate_hits_the_same_node() {
        // A coordinate is hit-tested by the RUNTIME, not by the resolver — so it
        // must land on exactly the node a test-id names.
        let (mut runtime, clicks) = runtime();
        let by_id = SyntheticDispatcher::new()
            .dispatch(
                &mut runtime,
                &SyntheticInput::pointer_move(SelectorTarget::test_id("submit")),
            )
            .target
            .expect("the test-id resolved");

        let point = Point::new(4.0, 4.0);
        let outcomes = SyntheticDispatcher::new().dispatch_all(
            &mut runtime,
            &[
                SyntheticInput::pointer_down(
                    SelectorTarget::coordinate(point),
                    PointerButton::Primary,
                ),
                SyntheticInput::pointer_up(
                    SelectorTarget::coordinate(point),
                    PointerButton::Primary,
                ),
            ],
        );
        assert_eq!(
            outcomes[0].target,
            Some(by_id),
            "same node by either selector"
        );
        assert_eq!(clicks.get(), 1);
    }

    #[test]
    fn the_propagation_path_runs_from_the_target_to_the_root() {
        let (mut runtime, _) = runtime();
        let outcome = SyntheticDispatcher::new().dispatch(
            &mut runtime,
            &SyntheticInput::pointer_move(SelectorTarget::test_id("submit")),
        );
        let root = runtime.tree().root().expect("a root");
        let target = outcome.target.expect("resolved");

        assert_eq!(
            outcome.propagation_path.first(),
            Some(&target),
            "starts at the target"
        );
        assert_eq!(
            outcome.propagation_path.last(),
            Some(&root),
            "ends at the root"
        );
        assert!(outcome.propagates_through(root));
    }

    #[test]
    fn an_unknown_test_id_dispatches_nothing_rather_than_guessing() {
        let (mut runtime, clicks) = runtime();
        let outcome = SyntheticDispatcher::new().dispatch(
            &mut runtime,
            &SyntheticInput::pointer_down(SelectorTarget::test_id("nope"), PointerButton::Primary),
        );
        assert!(!outcome.resolved());
        assert_eq!(outcome.propagation_path, Vec::new());
        assert_eq!(clicks.get(), 0, "no command ran");
    }

    #[test]
    fn a_move_reports_the_hover_change_it_caused() {
        let (mut runtime, _) = runtime();
        let outcome = SyntheticDispatcher::new().dispatch(
            &mut runtime,
            &SyntheticInput::pointer_move(SelectorTarget::test_id("submit")),
        );
        assert!(outcome.hover_changed, "moving onto the node entered it");

        // Moving onto the same node again changes nothing.
        let again = SyntheticDispatcher::new().dispatch(
            &mut runtime,
            &SyntheticInput::pointer_move(SelectorTarget::test_id("submit")),
        );
        assert!(!again.hover_changed);
    }

    #[test]
    fn key_input_is_reported_unresolved_rather_than_faked() {
        // There is no keycode → KeyEvent map yet. Saying so is honest; inventing
        // a key would make a headless test pass against nothing.
        let (mut runtime, _) = runtime();
        let outcome =
            SyntheticDispatcher::new().dispatch(&mut runtime, &SyntheticInput::key_down(42));
        assert!(!outcome.resolved());
    }
}
