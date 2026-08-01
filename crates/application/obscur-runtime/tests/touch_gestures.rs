#![doc = include_str!("touch_gestures.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_geometry::Point;
use obscur_identity::{ComponentId, InternedSymbol, NodeId};
use obscur_interaction::pointer::{PointerEventKind, PointerId};
use obscur_runtime::UiRuntime;
use obscur_style::cascade::CascadeOrigin;
use obscur_style::matching::Stylesheet;
use obscur_style::selector::PseudoState;
use obscur_testkit::fixture::{
    ClassRule, InteractionPanel, LongPressPanel, ScrollingList, TestFixture, finger, finger_at,
    mouse, untrusted_finger,
};
use obscur_view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

/// Two fingers, two controls, two independent presses.
///
/// This is the whole point of keying presses by pointer. With a single anchor
/// the second contact overwrote the first, so the *first* finger's release
/// looked up the second finger's target, matched nothing, and activated
/// nothing — one of the two buttons simply never fired, and which one
/// depended on the order the fingers happened to land.
#[test]
fn two_fingers_hold_two_controls_and_each_release_activates_its_own() {
    let cmd_a = InteractionPanel::CMD_A;
    let cmd_b = InteractionPanel::CMD_B;
    let mut runtime = TestFixture::runtime();
    let (ran_a, ran_b) = (Rc::new(Cell::new(false)), Rc::new(Cell::new(false)));
    {
        let (a, b) = (ran_a.clone(), ran_b.clone());
        runtime.on_command(cmd_a, move || a.set(true));
        runtime.on_command(cmd_b, move || b.set(true));
    }
    let sheet = InteractionPanel::sheet();
    let _ = runtime.render_now(
        &InteractionPanel::command_view(cmd_a, cmd_b),
        std::slice::from_ref(&sheet),
    );
    let (_panel, a, b) = InteractionPanel::nodes(&runtime);

    // Both fingers go down before either lifts — the interleaving a single
    // anchor cannot represent.
    runtime.handle_pointer(finger(0, PointerEventKind::Down, 50.0, 25.0));
    runtime.handle_pointer(finger(1, PointerEventKind::Down, 50.0, 75.0));
    assert!(
        runtime.pseudo_state(a).contains(PseudoState::ACTIVE),
        "the control the first finger holds is `:active`"
    );
    assert!(
        runtime.pseudo_state(b).contains(PseudoState::ACTIVE),
        "and so is the one the second finger holds — both presses are \
         outstanding at once"
    );

    // Lift them in the order they landed; each release finds its own press.
    let first = runtime.handle_pointer(finger(0, PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(first.command(), Some(cmd_a));
    assert!(ran_a.get(), "the first finger's control ran its handler");
    assert!(!ran_b.get(), "and did not run the other's");
    assert!(
        !runtime.pseudo_state(a).contains(PseudoState::ACTIVE),
        "the lifted finger's control is no longer `:active`"
    );
    assert!(
        runtime.pseudo_state(b).contains(PseudoState::ACTIVE),
        "while the finger still down keeps its control `:active`"
    );

    let second = runtime.handle_pointer(finger(1, PointerEventKind::Up, 50.0, 75.0));
    assert_eq!(second.command(), Some(cmd_b));
    assert!(
        ran_b.get(),
        "the second finger's control ran its own handler"
    );
}

/// A cancelled contact activates nothing and leaves nothing held.
///
/// `Cancel` is what the system sends when it takes a sequence away — a shell
/// gesture claims it, the window loses the contact. Before touch there was no
/// producer of one, so `handle_pointer` ignored the kind entirely: the press
/// stayed outstanding and its `:active` stuck on the control forever.
#[test]
fn a_cancelled_contact_activates_nothing_and_ends_the_press() {
    let cmd_a = InteractionPanel::CMD_A;
    let cmd_b = InteractionPanel::CMD_B;
    let mut runtime = TestFixture::runtime();
    let ran = Rc::new(Cell::new(false));
    {
        let r = ran.clone();
        runtime.on_command(cmd_a, move || r.set(true));
    }
    let sheet = InteractionPanel::sheet();
    let _ = runtime.render_now(
        &InteractionPanel::command_view(cmd_a, cmd_b),
        std::slice::from_ref(&sheet),
    );
    let (_panel, a, _b) = InteractionPanel::nodes(&runtime);

    runtime.handle_pointer(finger(0, PointerEventKind::Down, 50.0, 25.0));
    assert!(runtime.pseudo_state(a).contains(PseudoState::ACTIVE));

    let cancelled = runtime.handle_pointer(finger(0, PointerEventKind::Cancel, 50.0, 25.0));
    assert_eq!(
        cancelled.command(),
        None,
        "an interrupted gesture must not press what the finger rested on"
    );
    assert!(!ran.get(), "no handler runs for a cancelled contact");
    assert!(
        !runtime.pseudo_state(a).contains(PseudoState::ACTIVE),
        "the cancelled press is released, not left holding the control"
    );

    // And it is genuinely gone: a later release for that contact — the shape a
    // platform can still deliver after a cancel — finds no press to complete.
    let late = runtime.handle_pointer(finger(0, PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(late.command(), None);
    assert!(!ran.get());
}

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
    flick_up(&mut runtime);
    let _ = runtime.run_due_frame_reactive();

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
        let _ = runtime.run_due_frame_reactive();
        steps += 1;
        assert!(steps < 1_000, "the coast terminated");
    }
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
