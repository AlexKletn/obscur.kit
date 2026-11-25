use super::*;

// ---- Pointer capture (§2.4) ------------------------------------------------

/// Capture overrides hit-testing — the property the whole mechanism exists
/// for. The pointer presses and releases squarely over `b`, and `a` gets the
/// gesture because `a` captured it.
#[test]
fn a_captured_pointer_targets_its_captor_and_not_what_it_is_over() {
    let InteractionPanel {
        mut runtime,
        a,
        ran_a,
        ran_b,
        ..
    } = InteractionPanel::new();
    assert!(runtime.capture_pointer(PointerId::MOUSE, a));
    assert_eq!(runtime.pointer_capture(PointerId::MOUSE), Some(a));

    // (50, 75) is inside `b` and nowhere near `a`.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(dispatch.target(), Some(a), "the captor is the target");
    assert!(ran_a.get(), "the captor's own handler ran");
    assert!(
        !ran_b.get(),
        "the control actually under the pointer got nothing"
    );
}

/// §2.4: a capture ends when the pointer completes. Otherwise one drag would
/// swallow every gesture that followed it.
#[test]
fn a_completed_pointer_releases_its_capture() {
    let InteractionPanel {
        mut runtime,
        a,
        ran_b,
        ..
    } = InteractionPanel::new();
    runtime.capture_pointer(PointerId::MOUSE, a);
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(
        runtime.pointer_capture(PointerId::MOUSE),
        None,
        "the release ended the capture"
    );

    // The next gesture hit-tests normally and reaches what it is over.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    let dispatch = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));
    assert!(ran_b.get(), "the pointer is free again");
    assert!(dispatch.command().is_some());
}

/// A cancel completes the pointer too — the case that matters for touch,
/// where the system takes gestures away routinely.
#[test]
fn a_cancelled_pointer_releases_its_capture() {
    let InteractionPanel { mut runtime, a, .. } = InteractionPanel::new();
    runtime.capture_pointer(PointerId::touch(0), a);
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 50.0, 75.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Cancel, 50.0, 75.0));

    assert_eq!(runtime.pointer_capture(PointerId::touch(0)), None);
}

/// §2.4 releases a capture on node removal, and the runtime enforces it where
/// the capture is *used* rather than by hooking every removal path — so a
/// captor that leaves the tree cannot keep swallowing a pointer's events.
#[test]
fn a_capture_dies_with_its_node() {
    let InteractionPanel {
        mut runtime,
        a,
        ran_a,
        ran_b,
        ..
    } = InteractionPanel::new();
    runtime.capture_pointer(PointerId::MOUSE, a);

    // Re-render without `a`: `b` is the only child left, and it now occupies
    // the top half where `a` used to be. Binding the fixture's own command is
    // what makes `ran_b` below an answer.
    let cmd_b = InteractionPanel::CMD_B;
    let only_b = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(100))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(102))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                    .with_event(ViewEventBinding::new(EVENT_CLICK, cmd_b)),
            ])),
    ]));
    let sheet = InteractionPanel::sheet();
    let _ = runtime.render_now(&only_b, std::slice::from_ref(&sheet));

    assert_eq!(
        runtime.pointer_capture(PointerId::MOUSE),
        None,
        "a captor that has left the tree holds nothing"
    );

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert!(!ran_a.get(), "the removed node cannot still be activated");
    assert!(ran_b.get(), "the pointer hit-tests to what is really there");
}

/// Capturing to a node that is not in the tree captures nothing, rather than
/// parking a stale id that would swallow the pointer's events.
#[test]
fn capturing_an_unknown_node_captures_nothing() {
    let InteractionPanel {
        mut runtime, ran_b, ..
    } = InteractionPanel::new();
    assert!(!runtime.capture_pointer(PointerId::MOUSE, NodeId::from_raw(9_999)));
    assert_eq!(runtime.pointer_capture(PointerId::MOUSE), None);

    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 75.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 75.0));
    assert!(ran_b.get(), "routing is untouched");
}

/// Capture is per pointer ID: one captured finger leaves the others routing
/// by hit-test.
#[test]
fn capturing_one_pointer_leaves_the_others_alone() {
    let InteractionPanel {
        mut runtime,
        a,
        ran_a,
        ran_b,
        ..
    } = InteractionPanel::new();
    runtime.capture_pointer(PointerId::touch(0), a);

    // Both fingers work the same spot over `b`; only the captured one is
    // diverted to `a`.
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 50.0, 75.0));
    runtime.handle_pointer(finger(1, PointerEventKind::Down, 50.0, 75.0));
    let captured = runtime.handle_pointer(finger(0, PointerEventKind::Up, 50.0, 75.0));
    let free = runtime.handle_pointer(finger(1, PointerEventKind::Up, 50.0, 75.0));

    assert_eq!(captured.target(), Some(a));
    assert!(ran_a.get());
    assert!(free.target().is_some());
    assert!(
        ran_b.get(),
        "the uncaptured finger reached what it was over"
    );
}
