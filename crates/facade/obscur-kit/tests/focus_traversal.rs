#![doc = include_str!("focus_traversal.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};
use obscur_kit::interaction::event::UiEvent;
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::{
    CLASS, DISABLED, EVENT_KEY_DOWN, LOCAL_NAME, View, ViewEventBinding, ViewFragment, ViewNode,
    ViewProperty, ViewValue,
};

const ON_KEY: InternedSymbol = InternedSymbol::from_raw(0x7E01);

const CSS: &str = ".panel{width:300px;height:300px} .stop{width:300px;height:40px}";

fn compile(css: &str) -> Stylesheet {
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("focus_traversal.doc/stop.md")]
fn stop(local: &str) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(0x00D0_0001))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("stop".into())))
        .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::Text(local.into())))
}

#[doc = include_str!("focus_traversal.doc/committed.md")]
fn committed(kids: Vec<ViewNode>) -> (UiRuntime, Vec<NodeId>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let view = View::single(
        ViewNode::element(ComponentId::from_raw(0x00C0_0001))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(kids)),
    );
    let sheet = compile(CSS);
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));
    let root = rt.tree().root().expect("a root");
    let panel = rt.tree().get(root).expect("live").child_nodes()[0];
    let nodes = rt.tree().get(panel).expect("live").child_nodes().to_vec();
    (rt, nodes)
}

fn tab(shift: bool) -> KeyEvent {
    let key =
        KeyEvent::press(PhysicalKey::Tab, Some(LogicalKey::named(NamedKey::Tab))).as_trusted();
    if shift {
        key.with_modifiers(
            obscur_kit::interaction::command::Modifiers::new()
                .with(obscur_kit::interaction::command::Modifier::Shift),
        )
    } else {
        key
    }
}

#[doc = include_str!("focus_traversal.doc/tab_walks_the_controls_and_skips_everything_else.md")]
#[test]
fn tab_walks_the_controls_and_skips_everything_else() {
    // The hole this closes. `TabOrder` had existed in the domain with ZERO
    // callers anywhere in the workspace — not one, not even a test — so a
    // keyboard user could not reach a second control at all.
    let (mut rt, nodes) = committed(vec![stop("button"), stop("div"), stop("input")]);

    rt.handle_key(tab(false));
    assert_eq!(rt.focused(), Some(nodes[0]), "the first control");
    rt.handle_key(tab(false));
    assert_eq!(
        rt.focused(),
        Some(nodes[2]),
        "the div between them is not a stop"
    );
    rt.handle_key(tab(false));
    assert_eq!(rt.focused(), Some(nodes[0]), "and the ring wraps");
}

#[doc = include_str!("focus_traversal.doc/shift_tab_walks_back.md")]
#[test]
fn shift_tab_walks_back() {
    let (mut rt, nodes) = committed(vec![stop("button"), stop("input")]);
    rt.handle_key(tab(false));
    assert_eq!(rt.focused(), Some(nodes[0]));
    rt.handle_key(tab(true));
    assert_eq!(rt.focused(), Some(nodes[1]), "backwards, wrapping");
}

#[doc = include_str!("focus_traversal.doc/a_disabled_control_is_not_a_stop.md")]
#[test]
fn a_disabled_control_is_not_a_stop() {
    let off = stop("button").with_prop(ViewProperty::new(DISABLED, ViewValue::Null));
    let (mut rt, nodes) = committed(vec![stop("button"), off, stop("input")]);
    rt.handle_key(tab(false));
    rt.handle_key(tab(false));
    assert_eq!(rt.focused(), Some(nodes[2]), "the disabled one is skipped");
}

#[doc = include_str!("focus_traversal.doc/keyboard_focus_shows_a_ring_and_a_press_does_not.md")]
#[test]
fn keyboard_focus_shows_a_ring_and_a_press_does_not() {
    // `:focus-visible` was modelled in the selector tier and set by NOTHING, so
    // the rule matched nothing and `spec_ch2_interaction`'s case asserting its
    // absence after a click passed for the wrong reason. Both halves are real
    // now.
    let (mut rt, nodes) = committed(vec![stop("button"), stop("input")]);

    rt.handle_key(tab(false));
    assert_eq!(rt.focused(), Some(nodes[0]));
    assert!(
        rt.is_focus_visible(),
        "a keyboard move shows the ring — otherwise the user cannot see where \
         Tab has taken them, which is the whole point of moving it"
    );
    assert!(
        rt.pseudo_state(nodes[0])
            .contains(PseudoState::FOCUS_VISIBLE)
    );

    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        rt.handle_pointer(
            PointerEvent::new(
                PointerId::MOUSE,
                PointerDevice::Mouse,
                kind,
                Point::new(10.0, 50.0),
            )
            .as_trusted(),
        );
    }
    assert_eq!(rt.focused(), Some(nodes[1]));
    assert!(!rt.is_focus_visible(), "and a press does not");
    assert!(
        !rt.pseudo_state(nodes[1])
            .contains(PseudoState::FOCUS_VISIBLE)
    );
    assert!(
        rt.pseudo_state(nodes[1]).contains(PseudoState::FOCUS),
        "`:focus` still matches — focus-visible is not an alias for focus"
    );
}

#[doc = include_str!("focus_traversal.doc/refusing_the_keydown_keeps_tab_for_the_application.md")]
#[test]
fn refusing_the_keydown_keeps_tab_for_the_application() {
    // The payoff of making `keydown` a real event: a grid or a code editor takes
    // Tab for itself, which is impossible if Tab is hard-wired.
    let bound = stop("button").with_event(ViewEventBinding::new(EVENT_KEY_DOWN, ON_KEY));
    let (mut rt, nodes) = committed(vec![bound, stop("input")]);
    rt.focus_node(Some(nodes[0]));
    assert_eq!(rt.focused(), Some(nodes[0]));

    let seen = Rc::new(Cell::new(0_u32));
    let s = Rc::clone(&seen);
    rt.on_command(ON_KEY, move |e: &mut UiEvent| {
        s.set(s.get() + 1);
        e.prevent_default();
    });

    rt.handle_key(tab(false));
    assert_eq!(seen.get(), 1, "the handler saw the Tab");
    assert_eq!(
        rt.focused(),
        Some(nodes[0]),
        "and refusing it kept focus where it was"
    );
}

#[doc = include_str!("focus_traversal.doc/a_programmatic_move_fires_the_pair_and_shows_no_ring.md")]
#[test]
fn a_programmatic_move_fires_the_pair_and_shows_no_ring() {
    // The setter that did not exist: focus had exactly one mover, the pointer
    // press, so an application could not put focus on a field it had revealed.
    let (mut rt, nodes) = committed(vec![stop("button"), stop("input")]);
    assert!(rt.focus_node(Some(nodes[1])));
    assert_eq!(rt.focused(), Some(nodes[1]));
    assert!(
        !rt.is_focus_visible(),
        "a programmatic move is not the user navigating by keyboard"
    );

    assert!(rt.focus_node(None), "and it can clear focus");
    assert_eq!(rt.focused(), None);
    assert!(
        !rt.focus_node(Some(NodeId::from_raw(9_999))),
        "a node the tree does not have is refused rather than quietly accepted"
    );
}

#[doc = include_str!("focus_traversal.doc/a_single_control_does_not_lose_focus_to_a_tab.md")]
#[test]
fn a_single_control_does_not_lose_focus_to_a_tab() {
    // "Nowhere to go" must not become "nowhere". A Tab that blurred the field
    // the user was typing in, because the screen holds one control, would be
    // worse than a Tab that does nothing.
    let (mut rt, nodes) = committed(vec![stop("input")]);
    rt.focus_node(Some(nodes[0]));
    rt.handle_key(tab(false));
    assert_eq!(rt.focused(), Some(nodes[0]));
}

#[doc = include_str!("focus_traversal.doc/focus_does_not_survive_the_node_it_was_on.md")]
#[test]
fn focus_does_not_survive_the_node_it_was_on() {
    // The defect: a structural frame could remove the focused node and nothing
    // noticed, so `focused` went on naming a dead id. It degraded quietly
    // wherever a dispatch looked the node up and found nothing — but the
    // accessibility projection is built from this field verbatim, so the
    // platform was told focus sat on a node that had ceased to exist.
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let panel = |kids: Vec<ViewNode>| {
        View::single(
            ViewNode::element(ComponentId::from_raw(0x00C0_0001))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(kids)),
        )
    };

    let _ = rt.render_now(&panel(vec![stop("button"), stop("input")]), sheets);
    rt.handle_key(tab(false));
    rt.handle_key(tab(false));
    let gone = rt.focused().expect("Tab reached the second control");

    // The same panel, one control shorter — the field the user was in is gone.
    // The first control keeps its id, so focus staying put would be invisible;
    // it is the SECOND that must not be named any more.
    let _ = rt.render_now(&panel(vec![stop("button")]), sheets);
    assert!(
        rt.tree().get(gone).is_none(),
        "the node the user was on really is gone from the tree"
    );
    assert_eq!(
        rt.focused(),
        None,
        "so focus is cleared rather than left naming a dead id"
    );
}
