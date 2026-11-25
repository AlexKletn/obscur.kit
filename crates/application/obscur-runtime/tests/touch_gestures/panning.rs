use super::*;

// ---- Touch panning (§6.16 `Pan`) -------------------------------------------

#[test]
fn a_finger_dragging_a_list_scrolls_it_one_to_one() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 100.0, 60.0));
    // Up the screen by 40px: past the slop, so the pan begins and the content
    // follows the finger — what is below comes into view.
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 20.0));

    assert!(
        (runtime.scroll_top(scroller) - 40.0).abs() < f32::EPSILON,
        "the whole 40px of travel scrolled the list, with no slop swallowed; \
         got {}",
        runtime.scroll_top(scroller)
    );

    // And it keeps tracking: another 20px up.
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 0.0));
    assert!((runtime.scroll_top(scroller) - 60.0).abs() < f32::EPSILON);

    // Dragging back down reverses it.
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 30.0));
    assert!((runtime.scroll_top(scroller) - 30.0).abs() < f32::EPSILON);
}

/// The behaviour that makes touch scrolling usable at all: flicking a list
/// must not press the row the finger happened to land on.
#[test]
fn a_flick_does_not_activate_the_row_it_started_on() {
    let ScrollingList {
        mut runtime,
        scroller,
        tapped: ran,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 100.0, 60.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 20.0));
    let dispatch = runtime.handle_pointer(finger(0, PointerEventKind::Up, 100.0, 20.0));

    assert!(runtime.scroll_top(scroller) > 0.0, "the list did scroll");
    assert_eq!(
        dispatch.command(),
        None,
        "the release that ends a scroll is not a click"
    );
    assert!(!ran.get(), "no row handler ran");
}

/// Flicks the list upward: three 30px samples 30ms apart, then lifts. That is
/// 1000 px/s, so the coast should be about `1000 × FLING_TAU` = 325px — but
/// the list only has 120px of scroll, so it will arrive at the end.
fn flick_up(runtime: &mut UiRuntime) {
    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, 100.0, 110.0, 0));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Move, 100.0, 80.0, 30_000));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Move, 100.0, 50.0, 60_000));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Up, 100.0, 50.0, 60_000));
}

#[test]
fn a_flick_keeps_the_list_moving_after_the_finger_lifts() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    flick_up(&mut runtime);
    let at_release = runtime.scroll_top(scroller);

    runtime.advance_motion(0.016);
    let after_a_frame = runtime.scroll_top(scroller);
    assert!(
        after_a_frame > at_release,
        "the list coasted on: {at_release} → {after_a_frame}"
    );

    // And it comes to rest instead of coasting forever.
    for _ in 0..600 {
        runtime.advance_motion(0.016);
    }
    let settled = runtime.scroll_top(scroller);
    runtime.advance_motion(0.016);
    assert!(
        (runtime.scroll_top(scroller) - settled).abs() < f32::EPSILON,
        "the coast ended"
    );
}

/// The pair to the test above: a drag that ends slowly has *placed* the list,
/// not thrown it, and nothing should keep moving.
#[test]
fn a_slow_drag_does_not_fling() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    // 30px over a full second — deliberate, not a flick.
    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, 100.0, 110.0, 0));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Move, 100.0, 80.0, 1_000_000));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Up, 100.0, 80.0, 1_000_000));
    let at_release = runtime.scroll_top(scroller);

    runtime.advance_motion(0.016);
    assert!(
        (runtime.scroll_top(scroller) - at_release).abs() < f32::EPSILON,
        "a placed list stays where it was put"
    );
}

/// A finger that stops and *then* lifts has thrown nothing, however fast it
/// was moving a moment before. No events arrive while a finger rests, so the
/// last sample still holds the old speed — this is the case a naive velocity
/// gets wrong, and it is the one users notice, because stopping a list with
/// your finger is deliberate.
#[test]
fn a_finger_that_stops_before_lifting_does_not_fling() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, 100.0, 110.0, 0));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Move, 100.0, 80.0, 30_000));
    runtime.handle_pointer(finger_at(0, PointerEventKind::Move, 100.0, 50.0, 60_000));
    // Held still for a quarter second, then lifted.
    runtime.handle_pointer(finger_at(0, PointerEventKind::Up, 100.0, 50.0, 310_000));
    let at_release = runtime.scroll_top(scroller);

    runtime.advance_motion(0.016);
    assert!(
        (runtime.scroll_top(scroller) - at_release).abs() < f32::EPSILON,
        "the list stayed where the finger left it"
    );
}

#[test]
fn touching_a_coasting_list_stops_it() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    flick_up(&mut runtime);
    runtime.advance_motion(0.016);
    let moving = runtime.scroll_top(scroller);

    // Catch it.
    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, 100.0, 60.0, 500_000));
    runtime.advance_motion(0.016);
    assert!(
        (runtime.scroll_top(scroller) - moving).abs() < f32::EPSILON,
        "the list stopped under the finger"
    );
}

/// And the contact that caught it is spent doing so — it must not also open
/// whatever row was sliding past at that instant.
#[test]
fn the_tap_that_catches_a_coasting_list_activates_nothing() {
    let ScrollingList {
        mut runtime,
        tapped: ran,
        ..
    } = ScrollingList::new();
    flick_up(&mut runtime);
    runtime.advance_motion(0.016);
    assert!(!ran.get(), "the flick itself activated nothing");

    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, 100.0, 60.0, 500_000));
    let dispatch = runtime.handle_pointer(finger_at(0, PointerEventKind::Up, 100.0, 60.0, 560_000));

    assert_eq!(dispatch.command(), None);
    assert!(!ran.get(), "catching a moving list is not tapping a row");
}

/// A tap on a list that is *not* moving is an ordinary tap — otherwise the
/// rule above would have quietly disabled tapping altogether.
#[test]
fn a_tap_on_a_still_list_activates_its_row() {
    let ScrollingList {
        mut runtime,
        tapped: ran,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger_at(0, PointerEventKind::Down, 100.0, 60.0, 0));
    let dispatch = runtime.handle_pointer(finger_at(0, PointerEventKind::Up, 100.0, 60.0, 50_000));

    assert!(dispatch.command().is_some());
    assert!(ran.get());
}

/// Every step of a coast — including the one that ends it — has to reach
/// paint. A fling that stopped requesting frames on its final step would
/// leave the list a few pixels short of where it actually came to rest, until
/// something unrelated happened to schedule a frame.
#[test]
fn every_step_of_a_coast_asks_for_the_frame_that_paints_it() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    let view = ScrollingList::view_with_click(ScrollingList::CMD_ROW);
    let sheets = [ScrollingList::sheet()];
    flick_up(&mut runtime);
    let _ = runtime.run_due_frame(&view, &sheets);

    let at_release = runtime.scroll_top(scroller);
    for delta in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        runtime.advance_motion(delta);
        assert_eq!(runtime.scroll_top(scroller), at_release);
    }

    let mut steps = 0;
    loop {
        let before = runtime.scroll_top(scroller);
        runtime.advance_motion(0.016);
        let moved = (runtime.scroll_top(scroller) - before).abs() > f32::EPSILON;
        if !moved {
            break;
        }
        assert!(
            runtime.has_pending_frame(),
            "step {steps} moved the list and must be painted"
        );
        let _ = runtime.run_due_frame(&view, &sheets);
        steps += 1;
        assert!(steps < 1_000, "the coast terminated");
    }
    assert!(steps > 0, "the fling still moves on valid ticks");
    let settled = runtime.scroll_top(scroller);
    assert!(settled.is_finite());
    for _ in 0..120 {
        let _ = runtime.run_due_frame(&view, &sheets);
        runtime.advance_motion(0.016);
        if !runtime.has_pending_frame() {
            break;
        }
    }
    assert_eq!(runtime.scroll_top(scroller), settled);
    assert!(
        !runtime.has_pending_frame(),
        "the coast stops asking for frames"
    );
}

#[test]
fn a_coast_stops_at_the_end_of_the_content() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    flick_up(&mut runtime);
    // 1000 px/s coasts ~325px, and there are only 120px of scroll.
    for _ in 0..120 {
        runtime.advance_motion(0.016);
    }
    assert!(
        (runtime.scroll_top(scroller) - 120.0).abs() < f32::EPSILON,
        "the list is at its end, clamped; got {}",
        runtime.scroll_top(scroller)
    );
}

/// The case the *press discard* exists for, as opposed to the capture.
///
/// A flick that started on a row is already saved by the capture: the release
/// targets the container, which is not the row that was pressed, so nothing
/// matches and nothing activates. But when the drag starts on the scroll
/// container **itself** — bare space beside the rows — the pressed node and
/// the captor are the same node, the release matches, and the container's own
/// binding fires at the end of every flick. Only discarding the press stops
/// that.
#[test]
fn a_flick_that_starts_on_the_container_does_not_activate_the_container() {
    let cmd = InternedSymbol::from_raw(0x0D);
    let mut runtime = TestFixture::runtime();
    let ran = Rc::new(Cell::new(false));
    {
        let r = ran.clone();
        runtime.on_command(cmd, move || r.set(true));
    }
    // Rows are half the scroller's width, so x = 150 is inside the container
    // and on no row at all.
    let sheet = Stylesheet::new(
        CascadeOrigin::GlobalApp,
        vec![
            ClassRule::new("scroller")
                .sized(200.0, 120.0)
                .scrollable()
                .build(),
            ClassRule::new("row").sized(100.0, 40.0).build(),
        ],
    );
    let rows: Vec<ViewNode> = (0..6)
        .map(|i| {
            ViewNode::element(ComponentId::from_raw(200 + i))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        })
        .collect();
    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(100))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("scroller".into())))
            .with_event(ViewEventBinding::new(EVENT_CLICK, cmd))
            .with_children(ViewFragment::from_nodes(rows)),
    ]));
    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));
    let (scroller_node, _rows) = ScrollingList::nodes(&runtime);

    runtime.handle_pointer(finger(0, PointerEventKind::Down, 150.0, 60.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 150.0, 20.0));
    let dispatch = runtime.handle_pointer(finger(0, PointerEventKind::Up, 150.0, 20.0));

    assert!(
        runtime.scroll_top(scroller_node) > 0.0,
        "the drag scrolled the container it started on"
    );
    assert_eq!(
        dispatch.command(),
        None,
        "and the release that ended it is not a click on that container"
    );
    assert!(!ran.get(), "the container's own handler did not run");
}

/// The same gesture, one axis different — the finger stays inside the slop —
/// and it is a tap again. Without this pair, the test above would also pass
/// for an implementation that simply broke activation.
#[test]
fn a_contact_that_stays_inside_the_slop_still_taps_the_row() {
    let ScrollingList {
        mut runtime,
        scroller,
        tapped: ran,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 100.0, 60.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 57.0));
    let dispatch = runtime.handle_pointer(finger(0, PointerEventKind::Up, 100.0, 57.0));

    assert!(
        (runtime.scroll_top(scroller) - 0.0).abs() < f32::EPSILON,
        "a tap does not nudge the list"
    );
    assert!(dispatch.command().is_some(), "the row was tapped");
    assert!(ran.get(), "the row's handler ran");
}

/// A mouse drag is a selection or a data drag, not a scroll — the mouse
/// already has the wheel. Only a *direct* contact drags content.
#[test]
fn a_mouse_drag_does_not_scroll_the_list() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(mouse(PointerEventKind::Down, 100.0, 60.0));
    runtime.handle_pointer(mouse(PointerEventKind::Move, 100.0, 20.0));

    assert!(
        (runtime.scroll_top(scroller) - 0.0).abs() < f32::EPSILON,
        "the mouse must not drag the content"
    );

    // The wheel still scrolls the same container, so this is a gate on the
    // gesture and not a broken scroller.
    runtime.handle_scroll(30.0, Point::new(100.0, 60.0));
    assert!((runtime.scroll_top(scroller) - 30.0).abs() < f32::EPSILON);
}

/// A drag belongs to the list it started in. Once the finger has left the
/// scroller's bounds the content under it says nothing about what should
/// scroll — the remembered container does.
#[test]
fn a_pan_keeps_scrolling_its_own_list_after_the_finger_leaves_it() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 100.0, 60.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 20.0));
    // Well outside the 200×120 scroller.
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, -10.0));

    assert!(
        (runtime.scroll_top(scroller) - 70.0).abs() < f32::EPSILON,
        "the drag went on scrolling the list it started in; got {}",
        runtime.scroll_top(scroller)
    );
}

/// While a pan is in flight the pointer is captured by the container, so the
/// finger sweeping across the list does not light up every row it crosses.
#[test]
fn a_panning_finger_targets_its_container_and_hovers_nothing_it_sweeps_over() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 100.0, 60.0));
    let moved = runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 20.0));

    assert_eq!(
        runtime.pointer_capture(PointerId::touch(0)),
        Some(scroller),
        "the container captured the pointer when the pan began"
    );
    assert_eq!(
        moved.target(),
        Some(scroller),
        "and the events target it, not the row under the finger"
    );
}

/// A cancelled contact ends the pan with it — the finger is gone, so nothing
/// should still be following it.
#[test]
fn a_cancelled_contact_ends_the_pan() {
    let ScrollingList {
        mut runtime,
        scroller,
        ..
    } = ScrollingList::new();
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 100.0, 60.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 20.0));
    runtime.handle_pointer(finger(0, PointerEventKind::Cancel, 100.0, 20.0));
    let at = runtime.scroll_top(scroller);

    // A stray move for a contact that is over changes nothing.
    runtime.handle_pointer(finger(0, PointerEventKind::Move, 100.0, 0.0));
    assert!((runtime.scroll_top(scroller) - at).abs() < f32::EPSILON);
    assert_eq!(runtime.pointer_capture(PointerId::touch(0)), None);
}
