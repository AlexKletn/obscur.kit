use super::super::{BackGesturePhase, GestureSource};
use super::*;
use crate::pointer::{PointerDevice, PointerEventKind, PointerId};
use obscur_geometry::Point;

fn touch(kind: PointerEventKind, x: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::touch(1),
        PointerDevice::Touch,
        kind,
        Point::new(x, 40.0),
    )
    .as_trusted()
}

#[test]
fn a_touch_reports_ordered_phases_and_releases_its_offset() {
    let mut gesture = BackGesture::default();
    assert_eq!(
        gesture.update_touch(touch(PointerEventKind::Down, 12.0)),
        None
    );
    for (kind, x, phase, offset) in [
        (
            PointerEventKind::Move,
            24.0,
            BackGesturePhase::Started,
            12.0,
        ),
        (
            PointerEventKind::Move,
            50.0,
            BackGesturePhase::Updated,
            38.0,
        ),
        (
            PointerEventKind::Up,
            100.0,
            BackGesturePhase::Completed,
            88.0,
        ),
    ] {
        assert_eq!(
            gesture.update_touch(touch(kind, x)),
            Some(BackGestureUpdate {
                source: GestureSource::Touchscreen,
                phase,
                offset
            })
        );
    }
    assert_eq!(gesture.offset(), None);
    assert_eq!(gesture.cancel(), None);
}

#[test]
fn native_cancellation_ends_a_recognized_contact_without_completing_it() {
    let mut gesture = BackGesture::default();
    gesture.update_touch(touch(PointerEventKind::Down, 4.0));
    gesture.update_touch(touch(PointerEventKind::Move, 100.0));
    assert_eq!(
        gesture.update_touch(touch(PointerEventKind::Cancel, 110.0)),
        Some(BackGestureUpdate {
            source: GestureSource::Touchscreen,
            phase: BackGesturePhase::Cancelled,
            offset: 96.0
        }),
    );
    assert_eq!(
        gesture.update_touch(touch(PointerEventKind::Up, 120.0)),
        None
    );
}

#[test]
fn native_back_swipe_requires_the_configured_finger_count() {
    use super::super::{TouchpadGestureDelta, TouchpadGestureEvent};
    use crate::event::EventTrust;

    let mut gesture =
        BackGesture::default().with_touchpad_finger_count(NonZeroUsize::new(2).unwrap());
    for fingers in [0, 1, 3, 4, 2] {
        let mut completed = None;
        for (phase, x) in [
            (BackGesturePhase::Started, 0.0),
            (BackGesturePhase::Updated, 80.0),
            (BackGesturePhase::Completed, 0.0),
        ] {
            let update = gesture.update(
                &UiEvent::from_touchpad_gesture(TouchpadGestureEvent {
                    phase,
                    position: Point::new(32.0, 40.0),
                    finger_count: (phase == BackGesturePhase::Started)
                        .then(|| NonZeroUsize::new(fingers))
                        .flatten(),
                    delta: TouchpadGestureDelta::Swipe(Point::new(x, 0.0)),
                    trust: EventTrust::Trusted,
                    timestamp_micros: 0,
                }),
                1.0,
            );
            if fingers != 2 {
                assert!(update.is_none());
                assert_eq!(gesture.offset(), None);
            }
            if phase == BackGesturePhase::Completed {
                completed = update;
            }
        }
        assert_eq!(
            completed.map(|update| (update.source, update.phase)),
            (fingers == 2).then_some((GestureSource::Touchpad, BackGesturePhase::Completed))
        );
        assert_eq!(gesture.offset(), None);
    }
}

#[test]
fn native_touchpad_count_filter_preserves_touch_and_phased_scroll() {
    use crate::pointer::WheelPhase;

    let mut gesture =
        BackGesture::default().with_touchpad_finger_count(NonZeroUsize::new(2).unwrap());
    gesture.update_touch(touch(PointerEventKind::Down, 12.0));
    gesture.update_touch(touch(PointerEventKind::Move, 100.0));
    assert_eq!(
        gesture
            .update_touch(touch(PointerEventKind::Up, 100.0))
            .map(|event| event.phase),
        Some(BackGesturePhase::Completed),
    );

    let wheel = |phase, x| {
        WheelEvent::new(Point::new(0.0, 40.0), 0.0)
            .with_phase(phase)
            .with_delta_x(x)
            .as_trusted()
    };
    gesture.update_wheel(wheel(WheelPhase::Started, 0.0), 1.0);
    gesture.update_wheel(wheel(WheelPhase::Moved, -80.0), 1.0);
    assert_eq!(
        gesture
            .update_wheel(wheel(WheelPhase::Ended, 0.0), 1.0)
            .map(|event| event.phase),
        Some(BackGesturePhase::Completed),
    );
}

#[test]
fn a_rejected_native_start_cannot_complete_a_previous_swipe() {
    use super::super::{TouchpadGestureDelta, TouchpadGestureEvent};
    use crate::event::EventTrust;

    let mut gesture =
        BackGesture::default().with_touchpad_finger_count(NonZeroUsize::new(2).unwrap());
    let native = |phase, fingers, dx| {
        UiEvent::from_touchpad_gesture(TouchpadGestureEvent {
            phase,
            position: Point::new(32.0, 40.0),
            finger_count: NonZeroUsize::new(fingers),
            delta: TouchpadGestureDelta::Swipe(Point::new(dx, 0.0)),
            trust: EventTrust::Trusted,
            timestamp_micros: 0,
        })
    };
    gesture.update(&native(BackGesturePhase::Started, 2, 0.0), 1.0);
    gesture.update(&native(BackGesturePhase::Updated, 2, 80.0), 1.0);
    assert_eq!(gesture.offset(), Some(80.0));
    let interruption = gesture.update(&native(BackGesturePhase::Started, 3, 0.0), 1.0);
    assert_eq!(interruption.unwrap().phase, BackGesturePhase::Cancelled);
    assert_eq!(gesture.offset(), None);
    assert_eq!(
        gesture.update(&native(BackGesturePhase::Completed, 3, 0.0), 1.0),
        None,
    );
}
