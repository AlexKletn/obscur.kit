use super::*;

// ---- Long press (§6.16) ----------------------------------------------------

#[test]
fn a_contact_held_still_fires_the_long_press_before_it_lifts() {
    let LongPressPanel {
        mut runtime,
        clicked,
        held,
        ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));

    runtime.advance_motion(0.3);
    assert!(!held.get(), "not yet — the threshold is half a second");
    for delta in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        runtime.advance_motion(delta);
        assert!(!held.get(), "an invalid step cannot complete a hold");
    }
    runtime.advance_motion(0.3);
    assert!(held.get(), "the hold fired while the finger was still down");
    assert!(!clicked.get(), "and it is not a click");
}

/// The lift after a recognized hold must not *also* click. Holding something
/// and letting go does one thing, and it is the thing the user waited for.
#[test]
fn the_lift_after_a_long_press_is_not_a_click() {
    let LongPressPanel {
        mut runtime,
        clicked,
        held,
        ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(0.6);
    assert!(held.get());

    let dispatch = runtime.handle_pointer(finger(
        0,
        PointerEventKind::Up,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    assert_eq!(dispatch.command(), None);
    assert!(!clicked.get(), "the release was consumed by the hold");
}

/// The pair to the test above: lifting *before* the threshold is an ordinary
/// tap. Without this, an implementation that simply broke clicks on
/// long-pressable elements would pass.
#[test]
fn a_contact_that_lifts_before_the_threshold_is_a_plain_tap() {
    let LongPressPanel {
        mut runtime,
        clicked,
        held,
        ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(0.2);
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Up,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));

    assert!(clicked.get(), "the tap clicked");
    assert!(!held.get(), "and no hold fired");

    // And the hold cannot fire late, after the finger is long gone.
    runtime.advance_motion(1.0);
    assert!(!held.get());
}

/// A finger that wanders is scrolling, or dragging, or changing its mind —
/// anything but waiting. A context action firing mid-scroll is the failure
/// this prevents.
#[test]
fn a_contact_that_wanders_past_the_slop_fires_no_hold() {
    let LongPressPanel {
        mut runtime, held, ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(0.2);
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Move,
        50.0,
        LongPressPanel::HOLD_TARGET_Y - 30.0,
    ));
    runtime.advance_motion(1.0);

    assert!(!held.get(), "the hold was ruled out by the travel");
}

#[test]
fn jitter_within_the_slop_still_holds() {
    // The pair to the test above, one axis apart: a fingertip rolls while it
    // presses, and a hold any tremor killed would be unreachable.
    let LongPressPanel {
        mut runtime, held, ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(0.2);
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Move,
        52.0,
        LongPressPanel::HOLD_TARGET_Y + 2.0,
    ));
    runtime.advance_motion(0.4);

    assert!(held.get());
}

#[test]
fn a_cancelled_contact_fires_no_hold() {
    let LongPressPanel {
        mut runtime, held, ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(0.2);
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Cancel,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(1.0);

    assert!(!held.get());
}

/// A mouse has a second button for what a hold reaches, and a cursor resting
/// on a control is not a gesture at all — it is where the pointer happens to
/// be.
#[test]
fn a_resting_mouse_is_not_a_long_press() {
    let LongPressPanel {
        mut runtime, held, ..
    } = LongPressPanel::new();
    runtime.handle_pointer(mouse(
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    runtime.advance_motion(2.0);

    assert!(!held.get());
}

/// §2.3 step 4: the default action is gated on trust, exactly like a click's.
/// An injected contact is observed — the hold is counted — but the handler a
/// real finger would run does not run.
#[test]
fn an_untrusted_contact_counts_its_hold_but_runs_no_handler() {
    let LongPressPanel {
        mut runtime, held, ..
    } = LongPressPanel::new();
    let untrusted = untrusted_finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    );
    runtime.handle_pointer(untrusted);
    runtime.advance_motion(0.6);

    assert!(!held.get(), "a synthetic hold runs no gated default action");
}

/// A hold is the one gesture that fires while nothing is happening, so the
/// runtime has to keep asking for the frames that count it down — nothing
/// else will.
#[test]
fn an_armed_hold_keeps_asking_for_the_frames_that_count_it() {
    let LongPressPanel {
        mut runtime, held, ..
    } = LongPressPanel::new();
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));

    let mut ticks = 0;
    while !held.get() && ticks < 100 {
        assert!(
            runtime.has_pending_frame(),
            "tick {ticks}: the countdown must keep the loop turning"
        );
        let _ = runtime.run_due_frame_reactive();
        runtime.advance_motion(0.1);
        ticks += 1;
    }
    assert!(held.get(), "the hold fired within {ticks} ticks");
}

/// And an ordinary tap on something with no hold bound must not put the app
/// into a frame loop for half a second.
///
/// Measured by draining the scheduler and then ticking: a press schedules a
/// restyle frame either way, and that one is owed. What must differ is
/// whether *ticking* keeps producing more.
#[test]
fn a_contact_on_an_unbound_element_arms_nothing() {
    let LongPressPanel {
        mut runtime,
        view,
        sheet,
        clicked,
        ..
    } = LongPressPanel::new();
    let sheets = std::slice::from_ref(&sheet);

    // `a` (y ∈ [0,50)) binds only a click; `b` binds the hold.
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 50.0, 25.0));
    while runtime.run_due_frame(&view, sheets).is_some() {}
    runtime.advance_motion(0.1);
    assert!(
        !runtime.has_pending_frame(),
        "nothing is counting, so ticking schedules nothing"
    );

    runtime.handle_pointer(finger(0, PointerEventKind::Up, 50.0, 25.0));
    assert!(clicked.get(), "and it is still an ordinary tap");

    // The contrast, on the element that *does* bind a hold.
    runtime.handle_pointer(finger(
        0,
        PointerEventKind::Down,
        50.0,
        LongPressPanel::HOLD_TARGET_Y,
    ));
    while runtime.run_due_frame(&view, sheets).is_some() {}
    runtime.advance_motion(0.1);
    assert!(
        runtime.has_pending_frame(),
        "an armed hold keeps the loop turning"
    );
}
