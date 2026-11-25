#![doc = include_str!("touch_gestures.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "touch_gestures/long_press.rs"]
mod long_press;
#[path = "touch_gestures/panning.rs"]
mod panning;
#[path = "touch_gestures/pointer_capture.rs"]
mod pointer_capture;

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

#[test]
fn application_claim_and_scroll_pan_are_exclusive() {
    use std::cell::RefCell;

    use obscur_interaction::event::UiEvent;
    use obscur_view::view::EVENT_TOUCH_GESTURE;

    for app_claims in [false, true] {
        let mut runtime = TestFixture::runtime();
        let command = InternedSymbol::named("touch_candidate");
        let seen = Rc::new(RefCell::new(Vec::new()));
        let reported = Rc::clone(&seen);
        runtime.on_command(command, move |event: &mut UiEvent| {
            let kind = event.pointer().unwrap().kind();
            reported.borrow_mut().push(kind);
            if app_claims && kind == PointerEventKind::Move {
                event.prevent_default();
            }
        });
        let view = ScrollingList::with_rows(|_, row| {
            row.with_event(ViewEventBinding::new(EVENT_TOUCH_GESTURE, command))
        });
        runtime.render_now(&view, &[ScrollingList::sheet()]);
        let (scroller, _) = ScrollingList::nodes(&runtime);
        for (kind, y) in [
            (PointerEventKind::Down, 62.0),
            (PointerEventKind::Move, 44.0),
            (PointerEventKind::Move, 33.0),
            (PointerEventKind::Up, 30.0),
        ] {
            runtime.handle_pointer(finger(0, kind, 100.0, y));
        }
        let terminal = if app_claims {
            assert_eq!(runtime.scroll_top(scroller), 0.0);
            PointerEventKind::Up
        } else {
            assert!(runtime.scroll_top(scroller) > 0.0);
            PointerEventKind::Cancel
        };
        let seen = seen.borrow();
        assert_eq!(seen.first(), Some(&PointerEventKind::Down));
        assert_eq!(seen.last(), Some(&terminal));
        assert_eq!(seen.len(), if app_claims { 4 } else { 3 });
    }
}

#[test]
fn predictive_back_keeps_its_original_target_and_cancels_once() {
    use std::cell::RefCell;

    use obscur_interaction::event::UiEvent;
    use obscur_interaction::gesture::{GesturePhase, SystemBackEvent};
    use obscur_view::view::EVENT_SYSTEM_BACK;

    let mut runtime = TestFixture::runtime();
    let command = InternedSymbol::named("predictive_back");
    let original = Rc::new(RefCell::new(Vec::new()));
    let captured = Rc::clone(&original);
    runtime.on_command(command, move |event: &mut UiEvent| {
        assert!(event.trust.is_trusted());
        captured.borrow_mut().push(event.system_back().unwrap());
    });
    let view = ScrollingList::with_rows(|index, row| {
        if index == 1 {
            row.with_event(ViewEventBinding::new(EVENT_SYSTEM_BACK, command))
        } else {
            row
        }
    });
    runtime.render_now(&view, &[ScrollingList::sheet()]);
    let (_, rows) = ScrollingList::nodes(&runtime);
    let inputs = [
        (Some(rows[1]), GesturePhase::Started, 0.0),
        (Some(rows[0]), GesturePhase::Updated, 0.6),
        (None, GesturePhase::Cancelled, f32::NAN),
        (None, GesturePhase::Completed, 1.0),
    ];
    for (target, phase, progress) in inputs {
        runtime.handle_system_back(target, SystemBackEvent { phase, progress });
    }
    assert_eq!(
        *original.borrow(),
        [
            SystemBackEvent {
                phase: GesturePhase::Started,
                progress: 0.0
            },
            SystemBackEvent {
                phase: GesturePhase::Updated,
                progress: 0.6
            },
            SystemBackEvent {
                phase: GesturePhase::Cancelled,
                progress: 0.6
            },
        ]
    );
}

#[test]
fn phased_scroll_keeps_one_winner_after_the_axis_changes() {
    use std::cell::RefCell;

    use obscur_interaction::event::UiEvent;
    use obscur_interaction::pointer::{WheelEvent, WheelPhase};
    use obscur_view::view::EVENT_SCROLL_GESTURE;

    for horizontal in [false, true] {
        let mut runtime = TestFixture::runtime();
        let command = InternedSymbol::named("phased_candidate");
        let observed = Rc::new(RefCell::new(Vec::new()));
        let reported = Rc::clone(&observed);
        runtime.on_command(command, move |event: &mut UiEvent| {
            let wheel = event.wheel().unwrap();
            reported.borrow_mut().push(wheel.phase);
            if horizontal && wheel.delta_x.abs() >= 8.0 && wheel.phase == WheelPhase::Moved {
                event.prevent_default();
            }
        });
        let view = ScrollingList::with_rows(|index, row| {
            if index == 1 {
                row.with_event(ViewEventBinding::new(EVENT_SCROLL_GESTURE, command))
            } else {
                row
            }
        });
        runtime.render_now(&view, &[ScrollingList::sheet()]);
        let (scroller, _) = ScrollingList::nodes(&runtime);
        let (first_x, first_y) = if horizontal { (0.0, 1.0) } else { (0.0, 3.0) };
        let (second_x, second_y) = if horizontal { (-20.0, 2.0) } else { (1.0, 6.0) };
        let (third_x, third_y) = if horizontal {
            (0.0, 30.0)
        } else {
            (-40.0, 7.0)
        };
        for (phase, x, y, position) in [
            (
                WheelPhase::Started,
                first_x,
                first_y,
                Point::new(100.0, 62.0),
            ),
            (
                WheelPhase::Moved,
                second_x,
                second_y,
                Point::new(100.0, 62.0),
            ),
            (
                WheelPhase::Moved,
                third_x,
                third_y,
                Point::new(260.0, 140.0),
            ),
            (WheelPhase::Ended, 0.0, 0.0, Point::new(260.0, 140.0)),
        ] {
            runtime.handle_wheel(
                WheelEvent::new(position, y)
                    .with_delta_x(x)
                    .with_phase(phase)
                    .as_trusted(),
            );
        }
        if horizontal {
            assert_eq!(runtime.scroll_top(scroller), 0.0);
            assert_eq!(
                *observed.borrow(),
                [
                    WheelPhase::Started,
                    WheelPhase::Moved,
                    WheelPhase::Moved,
                    WheelPhase::Ended
                ]
            );
        } else {
            assert_eq!(runtime.scroll_top(scroller), 16.0);
            assert_eq!(
                *observed.borrow(),
                [
                    WheelPhase::Started,
                    WheelPhase::Moved,
                    WheelPhase::Cancelled
                ]
            );
        }
        let before_notch = runtime.scroll_top(scroller);
        runtime.handle_wheel(
            WheelEvent::new(Point::new(100.0, 62.0), 1.0)
                .in_lines()
                .as_trusted(),
        );
        assert_eq!(
            runtime.scroll_top(scroller),
            before_notch + UiRuntime::WHEEL_LINE_STEP
        );
    }
}

#[test]
fn native_touchpad_cancellation_reaches_the_original_binding_after_the_cursor_moves() {
    use std::cell::RefCell;

    use obscur_interaction::event::{EventTrust, UiEvent};
    use obscur_interaction::gesture::{GesturePhase, TouchpadGestureDelta, TouchpadGestureEvent};
    use obscur_view::view::EVENT_TOUCHPAD_GESTURE;

    let mut runtime = TestFixture::runtime();
    let command = InternedSymbol::named("native_candidate");
    let observed = Rc::new(RefCell::new(Vec::new()));
    let reported = Rc::clone(&observed);
    runtime.on_command(command, move |event: &mut UiEvent| {
        let gesture = event.touchpad_gesture().unwrap();
        reported
            .borrow_mut()
            .push((gesture.phase, gesture.position));
    });
    let view = ScrollingList::with_rows(|index, row| {
        if index == 1 {
            row.with_event(ViewEventBinding::new(EVENT_TOUCHPAD_GESTURE, command))
        } else {
            row
        }
    });
    runtime.render_now(&view, &[ScrollingList::sheet()]);
    let anchor = Point::new(100.0, 62.0);
    let mut gesture = TouchpadGestureEvent {
        phase: GesturePhase::Started,
        position: anchor,
        finger_count: std::num::NonZeroUsize::new(3),
        delta: TouchpadGestureDelta::Swipe(Point::new(0.0, 0.0)),
        trust: EventTrust::Trusted,
        timestamp_micros: 0,
    };
    runtime.handle_touchpad_gesture(gesture);
    gesture.phase = GesturePhase::Updated;
    gesture.position = Point::new(280.0, 160.0);
    runtime.handle_touchpad_gesture(gesture);
    runtime.cancel_indirect_gestures();
    gesture.phase = GesturePhase::Completed;
    runtime.handle_touchpad_gesture(gesture);
    assert_eq!(
        *observed.borrow(),
        [
            (GesturePhase::Started, anchor),
            (GesturePhase::Updated, anchor),
            (GesturePhase::Cancelled, anchor),
        ]
    );
}
