//! `SyntheticDispatcher`: replaying a [`SyntheticInput`] through the real input path.

use obscur_identity::NodeId;
use obscur_interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_runtime::UiRuntime;

use crate::input::{DispatchOutcome, SelectorResolver, SyntheticInput};

/// Replays a [`SyntheticInput`] against a live runtime and reports what it did
/// (spec §7.4).
///
/// This is the write half of headless testing. The descriptors and the resolver
/// only *describe* and *locate*; this drives.
///
/// # It uses the real path, deliberately
///
/// A dispatch is not a shortcut into the runtime's internals: a pointer input
/// becomes a [`PointerEvent`] aimed at a viewport point and goes through
/// [`UiRuntime::handle_pointer`] — the same call a windowed host makes from a
/// winit event. So hit-testing, the hover chain, focus movement and command
/// dispatch all behave exactly as they would for a real user, and a headless
/// assertion cannot pass while the real thing is broken. Aiming a `test-id` at
/// the centre of its box is the only synthesis; everything after is the engine.
///
/// This also generalises what every headless demo hand-rolled: locate a node,
/// compute a point inside it, build the event, call the runtime.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyntheticDispatcher {
    resolver: SelectorResolver,
}

impl SyntheticDispatcher {
    /// Creates a dispatcher.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            resolver: SelectorResolver::new(),
        }
    }

    /// Dispatches `input` against `runtime`, returning the typed outcome.
    ///
    /// An input whose target does not resolve — an unknown `test-id`, a node that
    /// is no longer live — is a no-op that reports
    /// [`DispatchOutcome::unresolved`]; it never panics and never guesses a
    /// different target.
    pub fn dispatch(self, runtime: &mut UiRuntime, input: &SyntheticInput) -> DispatchOutcome {
        match input {
            SyntheticInput::PointerMove { target } => {
                self.pointer(runtime, target, PointerEventKind::Move)
            }
            SyntheticInput::PointerDown { target, .. } => {
                self.pointer(runtime, target, PointerEventKind::Down)
            }
            SyntheticInput::PointerUp { target, .. } => {
                self.pointer(runtime, target, PointerEventKind::Up)
            }
            // Key injection needs a stable keycode → the runtime's structured
            // `KeyEvent` map, which does not exist yet. Reporting an unresolved
            // outcome is honest; synthesising a plausible key would not be.
            SyntheticInput::KeyDown { .. } | SyntheticInput::KeyUp { .. } => {
                DispatchOutcome::unresolved()
            }
        }
    }

    /// Dispatches every `input` in order, returning each outcome.
    ///
    /// Order matters and is preserved: a click is a `PointerDown` followed by a
    /// `PointerUp` on the same node, and the runtime only completes a click when
    /// it sees that pair.
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
    /// input path.
    fn pointer(
        self,
        runtime: &mut UiRuntime,
        target: &crate::input::SelectorTarget,
        kind: PointerEventKind,
    ) -> DispatchOutcome {
        let Some(point) = self.resolver.point_for(runtime, target) else {
            return DispatchOutcome::unresolved();
        };
        let event = PointerEvent::new(PointerId::MOUSE, PointerDevice::Mouse, kind, point);
        let dispatch = runtime.handle_pointer(event);

        DispatchOutcome {
            target: dispatch.target,
            propagation_path: dispatch
                .target
                .map(|node| ancestors_of(runtime, node))
                .unwrap_or_default(),
            command: dispatch.command,
            hover_changed: dispatch.hover_changed,
            focus_changed: dispatch.focus_changed,
        }
    }
}

/// The chain from `node` to the root, inclusive — the path an event's default
/// action is resolved along.
///
/// Walks the **logical** parent, which is the structural chain a command binding
/// is looked up through; a portalled node's visual parent is a paint concern and
/// would report a path events do not travel.
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
        CLASS, EVENT_CLICK, TEST_ID, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
        ViewValue,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    const SUBMIT: InternedSymbol = InternedSymbol::from_raw(0x9001);

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
                .with_event(ViewEventBinding::new(EVENT_CLICK, SUBMIT)),
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
