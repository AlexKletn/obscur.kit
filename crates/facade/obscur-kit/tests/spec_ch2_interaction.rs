#![doc = include_str!("spec_ch2_interaction.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch2_support/mod.rs"]
mod support;

use obscur_kit::geometry::Point;
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId};
use obscur_kit::interaction::event::{
    EventDispatcher, EventOrigin, EventPhase, EventPropagation, EventTrust, UiEvent,
};
use obscur_kit::interaction::pointer::{
    PointerCaptureRegistry, PointerDevice, PointerEvent, PointerEventKind, PointerId,
};
use obscur_kit::interaction::selection::{Selection, SelectionMode, SelectionPolicy};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::selector::PseudoState;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};
use support::{commit_styled, one_by_class, small_runtime};

#[doc = include_str!("spec_ch2_interaction.doc/node.md")]
fn node(raw: u64) -> NodeId {
    NodeId::from_raw(raw)
}

// ------------------------------------------------------------------- §2.3

#[doc = include_str!("spec_ch2_interaction.doc/propagation_is_capture_outermost_first_then_target_then_bubble_innermost_first.md")]
#[test]
fn propagation_is_capture_outermost_first_then_target_then_bubble_innermost_first() {
    let (a, b, c) = (node(1), node(2), node(3));
    let path = EventDispatcher::new().path(&[a, b, c]);
    let log: Vec<(NodeId, EventPhase)> = path.iter().map(|s| (s.node, s.phase)).collect();

    assert_eq!(
        log,
        vec![
            (a, EventPhase::Capture),
            (b, EventPhase::Capture),
            (c, EventPhase::Target),
            (b, EventPhase::Bubble),
            (a, EventPhase::Bubble),
        ],
        "capture strictly outermost-first, bubble strictly innermost-first"
    );
}

#[doc = include_str!("spec_ch2_interaction.doc/stop_propagation_and_stop_immediate_propagation_are_distinct_states.md")]
#[test]
fn stop_propagation_and_stop_immediate_propagation_are_distinct_states() {
    let mut stop = EventPropagation::new();
    stop.stop_propagation();

    let mut immediate = EventPropagation::new();
    immediate.stop_immediate_propagation();

    assert_ne!(
        stop, immediate,
        "`stop_propagation` (same-node listeners still run) and \
         `stop_immediate_propagation` (they do not) are different states"
    );
    assert!(
        immediate.is_stopped(),
        "an immediate stop also halts onward travel"
    );
}

#[doc = include_str!("spec_ch2_interaction.doc/prevent_default_and_stop_propagation_are_orthogonal.md")]
#[test]
fn prevent_default_and_stop_propagation_are_orthogonal() {
    let mut prevented = EventPropagation::new();
    prevented.prevent_default();
    assert!(
        !prevented.is_stopped(),
        "a listener calling prevent_default only must not stop propagation — \
         an ancestor's bubble listener still runs"
    );
    assert!(prevented.is_default_prevented());

    let mut stopped = EventPropagation::new();
    stopped.stop_propagation();
    assert!(
        !stopped.is_default_prevented(),
        "a listener calling only stop_propagation leaves the default action intact"
    );
}

#[doc = include_str!("spec_ch2_interaction.doc/target_and_current_target_differ_when_the_event_bubbles.md")]
#[test]
fn target_and_current_target_differ_when_the_event_bubbles() {
    // A > B > C, with the *click binding on A* — so a press landing on C must
    // resolve its handler at A.
    let mut rt = small_runtime();
    let command = InternedSymbol::from_raw(0x8000_0F01);
    let v = View::from_fragment(ViewFragment::from_nodes(vec![
        el("a", vec![el("b", vec![el("c", vec![])])])
            .with_event(ViewEventBinding::new(EVENT_CLICK, command)),
    ]));
    commit_styled(
        &mut rt,
        ".a{width:300px;height:300px} .b{width:200px;height:200px} \
         .c{width:100px;height:100px}",
        &v,
    );

    let a = one_by_class(&rt, "a").expect("A is retained");
    let c = one_by_class(&rt, "c").expect("C is retained");

    let dispatch = click(&mut rt, 10.0, 10.0, true);

    assert_eq!(dispatch.target(), Some(c), "`target` is C's node");
    assert_eq!(
        dispatch.current_target,
        Some(a),
        "`current_target` is A's node — two different values"
    );
}

#[doc = include_str!("spec_ch2_interaction.doc/a_synthetic_event_is_untrusted_and_a_gated_default_action_does_not_run.md")]
#[test]
fn a_synthetic_event_is_untrusted_and_a_gated_default_action_does_not_run() {
    let mut rt = small_runtime();
    let command = InternedSymbol::from_raw(0x8000_0F02);
    let v = View::from_fragment(ViewFragment::from_nodes(vec![
        el("btn", vec![]).with_event(ViewEventBinding::new(EVENT_CLICK, command)),
    ]));
    commit_styled(&mut rt, ".btn{width:100px;height:100px}", &v);

    // The platform-adapter input path.
    let trusted = click(&mut rt, 10.0, 10.0, true);
    assert_eq!(
        trusted.trust,
        EventTrust::Trusted,
        "the platform-originated event reports trusted = true"
    );
    assert_eq!(
        trusted.command,
        Some(command),
        "and its gated default action runs"
    );

    // The public dispatch API, programmatically.
    let synthetic = click(&mut rt, 10.0, 10.0, false);
    assert_eq!(
        synthetic.trust,
        EventTrust::Untrusted,
        "the programmatically dispatched one reports trusted = false"
    );
    assert_eq!(
        synthetic.command, None,
        "and its gated default action does NOT run — nothing consults the trust \
         flag today, so this is the assertion that keeps a real gap visible"
    );
}

// ------------------------------------------------------------------- §2.4

#[doc = include_str!("spec_ch2_interaction.doc/pointer_capture_is_per_pointer_id.md")]
#[test]
fn pointer_capture_is_per_pointer_id() {
    let mut registry = PointerCaptureRegistry::new();
    let (d, one, two) = (node(40), PointerId::from_raw(1), PointerId::from_raw(2));

    registry.capture(one, d);

    assert_eq!(
        registry.target(one),
        Some(d),
        "pointer 1's moves are delivered to D regardless of position"
    );
    assert_eq!(
        registry.target(two),
        None,
        "pointer 2 is unaffected — its move is delivered by normal hit-testing"
    );

    registry.release(one);
    assert_eq!(
        registry.target(one),
        None,
        "releasing pointer 1 releases it"
    );
    assert_eq!(
        registry.target(two),
        None,
        "and pointer 2 is still unaffected"
    );
}

#[doc = include_str!("spec_ch2_interaction.doc/a_touch_leaves_no_persistent_hover_after_release.md")]
#[test]
fn a_touch_leaves_no_persistent_hover_after_release() {
    let mut rt = small_runtime();
    let v = View::from_fragment(ViewFragment::from_nodes(vec![el("tap", vec![])]));
    commit_styled(
        &mut rt,
        ".tap{width:100px;height:100px;background-color:blue} \
         .tap:hover{background-color:red}",
        &v,
    );
    let tap = one_by_class(&rt, "tap").expect("the node is retained");

    let touch = PointerId::from_raw(7);
    press_release(&mut rt, touch, PointerDevice::Touch, 10.0, 10.0);

    assert!(
        !rt.pseudo_state(tap).contains(PseudoState::HOVER),
        "after the release the node is not in the hover state"
    );
    assert_eq!(
        rt.hovered(),
        None,
        "and nothing is left hovered by a touch that ended"
    );
}

// ------------------------------------------------------------------- §2.5

#[doc = include_str!("spec_ch2_interaction.doc/a_mouse_click_focuses_without_making_focus_visible.md")]
#[test]
fn a_mouse_click_focuses_without_making_focus_visible() {
    let mut rt = small_runtime();
    let v = View::from_fragment(ViewFragment::from_nodes(vec![el("btn", vec![])]));
    commit_styled(&mut rt, ".btn{width:100px;height:100px}", &v);
    let btn = one_by_class(&rt, "btn").expect("the button is retained");

    press_release(&mut rt, PointerId::MOUSE, PointerDevice::Mouse, 10.0, 10.0);

    assert_eq!(
        rt.focused(),
        Some(btn),
        "the button is focused (it receives keyboard events)"
    );
    assert!(
        !rt.pseudo_state(btn).contains(PseudoState::FOCUS_VISIBLE),
        "but the focus-visible rule does not apply to a pointer-driven focus — \
         focus-visible is not an alias for focus"
    );
}

// ------------------------------------------------------------------- §2.6

#[doc = include_str!("spec_ch2_interaction.doc/command_routing_visits_the_five_links_in_the_spec_s_order.md")]
#[test]
fn command_routing_visits_the_five_links_in_the_spec_s_order() {
    use obscur_kit::interaction::focus::{FocusScope, FocusScopeId};
    use obscur_kit::runtime::CommandTarget;

    let mut rt = small_runtime();
    let v = View::from_fragment(ViewFragment::from_nodes(vec![el(
        "outer",
        vec![el("inner", vec![])],
    )]));
    commit_styled(
        &mut rt,
        ".outer{width:200px;height:200px} .inner{width:100px;height:100px}",
        &v,
    );
    let outer = one_by_class(&rt, "outer").expect("C is retained");
    let inner = one_by_class(&rt, "inner").expect("N is retained");

    let scope = FocusScopeId::from_raw(1);
    rt.open_focus_scope(FocusScope::new(scope).with_member(inner));

    let route = rt.command_route(Some(inner));
    assert_eq!(
        route.first(),
        Some(&CommandTarget::Node(inner)),
        "1. focused node"
    );
    assert_eq!(
        route.get(1),
        Some(&CommandTarget::Scope(scope)),
        "2. focus scope — a distinct step, not folded into the ancestor walk"
    );
    assert!(
        route
            .iter()
            .any(|t| matches!(t, CommandTarget::Node(n) if *n == outer)),
        "3. ancestors"
    );
    assert_eq!(
        route.get(route.len().wrapping_sub(2)),
        Some(&CommandTarget::Window),
        "4. window"
    );
    assert_eq!(
        route.last(),
        Some(&CommandTarget::App),
        "5. application fallback"
    );
}

#[doc = include_str!("spec_ch2_interaction.doc/an_ime_composition_delivers_its_text_once_and_not_through_keydown.md")]
#[test]
fn an_ime_composition_delivers_its_text_once_and_not_through_keydown() {
    use obscur_kit::text::ime::CompositionEvent;

    let mut rt = small_runtime();
    let v = View::from_fragment(ViewFragment::from_nodes(vec![el("field", vec![])]));
    commit_styled(&mut rt, ".field{width:200px;height:40px}", &v);
    let field = one_by_class(&rt, "field").expect("the field is retained");

    rt.begin_text_editing(field, "");
    rt.handle_ime(CompositionEvent::Start);
    rt.handle_ime(CompositionEvent::Update(
        obscur_kit::text::ime::Preedit::new("か"),
    ));
    rt.handle_ime(CompositionEvent::Commit("が".into()));

    assert_eq!(
        rt.editing_text(),
        Some("が"),
        "the committed character arrives exactly once, through the \
         text-input/composition path — not once from a keydown and once from \
         the commit"
    );
    assert_eq!(
        rt.composition_preedit(),
        None,
        "and the composition session is over"
    );
}

// ------------------------------------------------------------------- §2.7

#[doc = include_str!("spec_ch2_interaction.doc/the_selection_anchor_persists_across_successive_shift_clicks.md")]
#[test]
fn the_selection_anchor_persists_across_successive_shift_clicks() {
    let policy = SelectionPolicy::new(SelectionMode::Multiple);
    let mut selection = Selection::new();
    let item = |i: u64| node(500 + i);

    // Click item 2.
    policy.apply_click(&mut selection, item(2), false);
    assert_eq!(selection.items().collect::<Vec<_>>(), vec![item(2)]);
    assert_eq!(
        selection.anchor().map(|a| a.anchor()),
        Some(item(2)),
        "anchor 2"
    );
    assert_eq!(selection.lead(), Some(item(2)), "active 2");

    // Shift+click item 5: the range runs from the anchor.
    policy.apply_range(&mut selection, &[item(2), item(3), item(4), item(5)]);
    let mut items: Vec<NodeId> = selection.items().collect();
    items.sort_unstable();
    assert_eq!(items, vec![item(2), item(3), item(4), item(5)]);
    assert_eq!(
        selection.anchor().map(|a| a.anchor()),
        Some(item(2)),
        "anchor still 2"
    );
    assert_eq!(selection.lead(), Some(item(5)), "active 5");

    // Shift+click item 3, with no intervening plain click: the range *shrinks*.
    policy.apply_range(&mut selection, &[item(2), item(3)]);
    let mut items: Vec<NodeId> = selection.items().collect();
    items.sort_unstable();
    assert_eq!(
        items,
        vec![item(2), item(3)],
        "selection is exactly {{2,3}} — items 4 and 5 are deselected, which only \
         a model with a distinct anchor can do"
    );
    assert_eq!(
        selection.anchor().map(|a| a.anchor()),
        Some(item(2)),
        "anchor is still 2"
    );
    assert_eq!(selection.lead(), Some(item(3)), "active is 3");
}

// ------------------------------------------------------------------ helpers

#[doc = include_str!("spec_ch2_interaction.doc/el.md")]
fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(container())
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

fn container() -> ComponentId {
    obscur_kit::ui_ir::name_binding::CONTAINER
}

#[doc = include_str!("spec_ch2_interaction.doc/press_release.md")]
fn press_release(rt: &mut UiRuntime, pointer: PointerId, device: PointerDevice, x: f32, y: f32) {
    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        let _ = rt.handle_pointer(
            PointerEvent::new(pointer, device, kind, Point::new(x, y)).as_trusted(),
        );
    }
}

#[doc = include_str!("spec_ch2_interaction.doc/click.md")]
fn click(
    rt: &mut UiRuntime,
    x: f32,
    y: f32,
    trusted: bool,
) -> obscur_kit::runtime::PointerDispatch {
    let mark = |e: PointerEvent| if trusted { e.as_trusted() } else { e };
    let _ = rt.handle_pointer(mark(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Down,
        Point::new(x, y),
    )));
    rt.handle_pointer(mark(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Up,
        Point::new(x, y),
    )))
}

#[doc = include_str!("spec_ch2_interaction.doc/_envelope.md")]
#[allow(dead_code)]
fn _envelope() -> UiEvent {
    UiEvent::new(EventOrigin::Synthetic, node(1))
}
