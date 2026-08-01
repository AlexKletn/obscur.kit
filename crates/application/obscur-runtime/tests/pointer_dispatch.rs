#![doc = include_str!("pointer_dispatch.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::pointer::PointerEventKind;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::Stylesheet;
use obscur_style::selector::PseudoState;
use obscur_testkit::fixture::{ClassRule, InteractionPanel, TestFixture, mouse};
use obscur_testkit::selector::NodePath;
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[test]
fn pointer_over_child_b_makes_it_the_hover_target() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (panel, a, b) = InteractionPanel::nodes(&runtime);

    // (50, 75) lands inside `b` (y∈[50,100)); the innermost node wins.
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
    assert_eq!(dispatch.target(), Some(b));
    assert!(dispatch.hover_changed());
    assert!(!dispatch.focus_changed());
    assert_eq!(runtime.hovered(), Some(b));

    // The hover chain records HOVER on b and its ancestors, not on a.
    assert!(runtime.pseudo_state(b).contains(PseudoState::HOVER));
    assert!(runtime.pseudo_state(panel).contains(PseudoState::HOVER));
    assert_eq!(runtime.pseudo_state(a), PseudoState::empty());
}

#[test]
fn press_sets_active_pseudo_and_release_clears_it() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (panel, _a, b) = InteractionPanel::nodes(&runtime);

    // A press marks the pressed node `b` and its ancestor `panel` `:active`
    // (CSS applies `:active` up the ancestor chain, like hover).
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    assert!(runtime.pseudo_state(b).contains(PseudoState::ACTIVE));
    assert!(runtime.pseudo_state(panel).contains(PseudoState::ACTIVE));

    // The matching release clears `:active` everywhere (hover/focus remain).
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));
    assert!(!runtime.pseudo_state(b).contains(PseudoState::ACTIVE));
    assert!(!runtime.pseudo_state(panel).contains(PseudoState::ACTIVE));
}

#[test]
fn disabled_control_takes_no_focus_and_fires_no_command() {
    use std::cell::Cell;
    use std::rc::Rc;

    use obscur_view::view::{DISABLED, EVENT_CLICK, TYPE, ViewEventBinding};

    // A single 100x50 `button` carrying the reserved DISABLED attribute and an
    // `on-click` binding. A native disabled control is inert.
    let cmd = InternedSymbol::from_raw(0x00BE_EF01);
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(200))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
            .with_prop(ViewProperty::new(TYPE, ViewValue::from("button")))
            .with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true)))
            .with_event(ViewEventBinding::new(EVENT_CLICK, cmd)),
    ]));
    let sheet = Stylesheet::new(
        CascadeOrigin::GlobalApp,
        vec![ClassRule::new("a").sized(100.0, 50.0).build()],
    );

    let mut runtime = TestFixture::runtime();
    let ran = Rc::new(Cell::new(false));
    {
        let r = ran.clone();
        runtime.on_command(cmd, move || r.set(true));
    }
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    // Press inside the disabled control: it takes NO focus.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    assert_eq!(
        runtime.focused(),
        None,
        "a disabled control must not take focus"
    );
    assert!(
        runtime.editing_text().is_none(),
        "a disabled control seats no editing session"
    );

    // Release on it activates NOTHING — no command reported, no handler ran.
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(
        dispatch.command(),
        None,
        "a disabled control reports no activation command"
    );
    assert!(!ran.get(), "a disabled control fires no activation command");

    // `:hover` is unaffected — CSS still hovers a disabled element.
    let node = NodePath::new(&[0]).resolve(&runtime);
    assert!(runtime.pseudo_state(node).contains(PseudoState::HOVER));
}

#[test]
fn hover_reaches_paint_end_to_end() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    // Cold frame: nothing is hovered, so no `:hover` background paints.
    let cold = runtime.render_now(&InteractionPanel::view(), sheets);
    let cold_fills = cold.snapshot().display_list().len();

    // Point at b, which schedules an Input frame; render it.
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
    assert!(dispatch.hover_changed());
    assert!(
        runtime.has_pending_frame(),
        "hover change requested a frame"
    );
    let hot = runtime
        .run_due_frame(&InteractionPanel::view(), sheets)
        .expect("the input-requested frame is due");

    // `.b:hover { background-color }` now contributes a fill that was absent
    // before hover — proving `:hover` matched through the runtime.
    assert!(
        hot.snapshot().display_list().len() > cold_fills,
        ":hover added a paint command once the pointer entered b"
    );
}

#[test]
fn moving_the_pointer_between_children_transitions_hover() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, b) = InteractionPanel::nodes(&runtime);

    // Enter b, then move to a: b leaves, a enters.
    let to_b = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
    assert_eq!(to_b.target(), Some(b));
    assert!(runtime.pseudo_state(b).contains(PseudoState::HOVER));

    let to_a = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0));
    assert_eq!(to_a.target(), Some(a));
    assert!(to_a.hover_changed());
    assert_eq!(runtime.hovered(), Some(a));
    // Hover moved: a is now HOVER, b is cleared.
    assert!(runtime.pseudo_state(a).contains(PseudoState::HOVER));
    assert_eq!(runtime.pseudo_state(b), PseudoState::empty());
}

#[test]
fn press_focuses_the_target_and_focus_reaches_paint() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let cold = runtime.render_now(&InteractionPanel::view(), sheets);
    let cold_fills = cold.snapshot().display_list().len();
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    // Press inside a → a takes focus.
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    assert_eq!(dispatch.target(), Some(a));
    assert!(dispatch.focus_changed());
    assert_eq!(runtime.focused(), Some(a));
    assert!(runtime.pseudo_state(a).contains(PseudoState::FOCUS));

    // `.a:focus { background-color }` now paints where it did not before.
    let focused = runtime
        .run_due_frame(&InteractionPanel::view(), sheets)
        .expect("the input-requested frame is due");
    assert!(
        focused.snapshot().display_list().len() > cold_fills,
        ":focus added a paint command once a was pressed"
    );
}

#[test]
fn hover_and_focus_bits_combine_on_the_same_node() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    // Hover a, then press a: it must carry HOVER *and* FOCUS.
    runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let state = runtime.pseudo_state(a);
    assert!(state.contains(PseudoState::HOVER));
    assert!(state.contains(PseudoState::FOCUS));
}

#[test]
fn pointer_outside_every_box_clears_hover() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);
    let (_panel, _a, b) = InteractionPanel::nodes(&runtime);

    // Hover b first.
    runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
    assert_eq!(runtime.hovered(), Some(b));

    // Move outside the viewport (root box is only 300x300) → no hits.
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 1000.0, 1000.0));
    assert_eq!(dispatch.target(), None);
    assert!(dispatch.hover_changed(), "leaving b is a hover change");
    assert_eq!(runtime.hovered(), None);
    assert_eq!(runtime.pseudo_state(b), PseudoState::empty());
}

#[test]
fn unchanged_hover_does_not_request_a_frame() {
    let mut runtime = TestFixture::runtime();
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);
    let _ = runtime.render_now(&InteractionPanel::view(), sheets);

    let first = runtime.handle_pointer(mouse(PointerEventKind::Move, 50.0, 75.0));
    assert!(first.hover_changed());
    // Drain the frame the first move scheduled.
    let _ = runtime.run_due_frame(&InteractionPanel::view(), sheets);
    assert!(!runtime.has_pending_frame());

    // A second move within the same node changes nothing → no frame.
    let second = runtime.handle_pointer(mouse(PointerEventKind::Move, 55.0, 80.0));
    assert!(!second.changed());
    assert!(!runtime.has_pending_frame());
}

#[test]
fn handle_pointer_is_safe_before_any_frame() {
    // No frame has run → the spatial index is empty; hit-testing must not
    // panic and must report nothing.
    let mut runtime = TestFixture::runtime();
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Move, 10.0, 10.0));
    assert_eq!(dispatch.target(), None);
    assert!(!dispatch.changed());
    assert_eq!(runtime.hovered(), None);
    assert_eq!(runtime.focused(), None);
}
