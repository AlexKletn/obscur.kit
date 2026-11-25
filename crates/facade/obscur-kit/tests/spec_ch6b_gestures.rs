#![doc = include_str!("spec_ch6b_gestures.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::identity::NodeId;
use obscur_kit::interaction::gesture::{
    GestureArena, GestureContender, GestureResolution, RecognizerId, RecognizerState,
};

#[doc = include_str!("spec_ch6b_gestures.doc/APP_PAN.md")]
const APP_PAN: u64 = 1;
#[doc = include_str!("spec_ch6b_gestures.doc/SYSTEM_BACK.md")]
const SYSTEM_BACK: u64 = 2;

fn node() -> NodeId {
    NodeId::from_raw(0x6B16)
}

#[doc = include_str!("spec_ch6b_gestures.doc/a_system_gesture_wins_the_arena_by_default.md")]
#[test]
fn a_system_gesture_wins_the_arena_by_default() {
    let app_pan = RecognizerId::from_raw(APP_PAN);
    let system_back = RecognizerId::from_raw(SYSTEM_BACK);

    let mut arena = GestureArena::new();
    arena.add(GestureContender::new(app_pan, node()));
    arena.add(GestureContender::new(system_back, node()));
    arena.mark(app_pan, RecognizerState::Recognized);
    arena.mark(system_back, RecognizerState::Recognized);

    assert_eq!(
        arena.sweep(),
        GestureResolution::Winner(system_back),
        "the reserved system back gesture must win by default against a \
         full-bleed application pan that neither declared nor was granted any \
         priority"
    );
}

#[doc = include_str!("spec_ch6b_gestures.doc/the_application_pan_is_cancelled_when_the_system_gesture_takes_the_sequence.md")]
#[test]
fn the_application_pan_is_cancelled_when_the_system_gesture_takes_the_sequence() {
    let app_pan = RecognizerId::from_raw(APP_PAN);
    let system_back = RecognizerId::from_raw(SYSTEM_BACK);

    let mut arena = GestureArena::new();
    arena.add(GestureContender::new(app_pan, node()));
    arena.add(GestureContender::new(system_back, node()));
    arena.mark(app_pan, RecognizerState::Recognized);
    arena.mark(system_back, RecognizerState::Recognized);
    let _ = arena.sweep();

    let app_state = arena
        .contenders()
        .iter()
        .find(|(c, _)| c.recognizer() == app_pan)
        .map(|(_, state)| *state)
        .expect("the application pan is still a tracked contender");

    assert_eq!(
        app_state,
        RecognizerState::Cancelled,
        "the application pan must be cancelled, not left recognized, once the \
         system back gesture owns the sequence"
    );
}

#[doc = include_str!("spec_ch6b_gestures.doc/an_application_pan_outside_a_reserved_region_wins.md")]
#[test]
fn an_application_pan_outside_a_reserved_region_wins() {
    let app_pan = RecognizerId::from_raw(APP_PAN);

    let mut arena = GestureArena::new();
    arena.add(GestureContender::new(app_pan, node()));
    arena.mark(app_pan, RecognizerState::Recognized);

    assert_eq!(
        arena.sweep(),
        GestureResolution::Winner(app_pan),
        "with no system gesture contending, the application pan wins regardless"
    );
}

#[test]
fn native_touchpad_swipe_preserves_source_and_count_through_dispatch_envelope() {
    use std::num::NonZeroUsize;

    use obscur_kit::geometry::Point;
    use obscur_kit::interaction::event::{EventTrust, UiEvent};
    use obscur_kit::interaction::gesture::{
        GesturePhase, GestureSource, SwipeDirection, SwipeGesture, TouchpadGestureDelta,
        TouchpadGestureEvent,
    };

    let count = NonZeroUsize::new(3).unwrap();
    let mut recognizer = SwipeGesture::in_direction(SwipeDirection::Right).with_finger_count(count);
    let mut event = TouchpadGestureEvent {
        phase: GesturePhase::Started,
        position: Point::new(40.0, 20.0),
        finger_count: Some(count),
        delta: TouchpadGestureDelta::Swipe(Point::new(0.0, 0.0)),
        trust: EventTrust::Trusted,
        timestamp_micros: 42,
    };
    let envelope = UiEvent::from_touchpad_gesture(event);
    assert_eq!(envelope.timestamp_micros, 42);
    assert_eq!(recognizer.update(&envelope, 2.0), None);

    event.phase = GesturePhase::Updated;
    event.delta = TouchpadGestureDelta::Swipe(Point::new(80.0, 0.0));
    let started = recognizer
        .update(&UiEvent::from_touchpad_gesture(event), 2.0)
        .unwrap();
    assert_eq!(started.phase, GesturePhase::Started);
    assert_eq!(started.source, GestureSource::Touchpad);
    assert_eq!(started.finger_count, Some(count));
    assert_eq!(started.offset, 80.0);

    event.phase = GesturePhase::Completed;
    event.finger_count = None;
    event.delta = TouchpadGestureDelta::Swipe(Point::new(0.0, 0.0));
    let completed = recognizer
        .update(&UiEvent::from_touchpad_gesture(event), 2.0)
        .unwrap();
    assert_eq!(completed.phase, GesturePhase::Completed);
    assert_eq!(completed.source, started.source);
    assert_eq!(completed.finger_count, started.finger_count);
}

#[test]
fn native_touchpad_transform_reports_scale_rotation_and_cancellation() {
    use std::num::NonZeroUsize;

    use obscur_kit::geometry::Point;
    use obscur_kit::interaction::event::{EventTrust, UiEvent};
    use obscur_kit::interaction::gesture::{
        GesturePhase, GestureSource, TouchpadGestureDelta, TouchpadGestureEvent, TransformGesture,
    };

    let center = Point::new(80.0, 40.0);
    let mut recognizer = TransformGesture::default();
    let mut event = TouchpadGestureEvent {
        phase: GesturePhase::Started,
        position: center,
        finger_count: NonZeroUsize::new(2),
        delta: TouchpadGestureDelta::Transform {
            scale: 1.0,
            rotation: 0.0,
        },
        trust: EventTrust::Trusted,
        timestamp_micros: 42,
    };
    let started = recognizer
        .update(&UiEvent::from_touchpad_gesture(event))
        .unwrap();
    assert_eq!(started.phase, GesturePhase::Started);
    assert_eq!(started.source, GestureSource::Touchpad);
    assert_eq!(started.center, center);

    event.phase = GesturePhase::Updated;
    event.delta = TouchpadGestureDelta::Transform {
        scale: 1.5,
        rotation: 0.25,
    };
    let updated = recognizer
        .update(&UiEvent::from_touchpad_gesture(event))
        .unwrap();
    assert_eq!(updated.phase, GesturePhase::Updated);
    assert_eq!(updated.scale, 1.5);
    assert_eq!(updated.rotation, 0.25);
    assert_eq!(updated.finger_count, event.finger_count);

    event.phase = GesturePhase::Cancelled;
    let cancelled = recognizer
        .update(&UiEvent::from_touchpad_gesture(event))
        .unwrap();
    assert_eq!(cancelled.phase, GesturePhase::Cancelled);
    assert_eq!(cancelled.source, GestureSource::Touchpad);
    assert_eq!(
        recognizer.update(&UiEvent::from_touchpad_gesture(event)),
        None
    );
}
