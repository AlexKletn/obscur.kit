//! Recognizer for a semantic back swipe.

use crate::event::UiEvent;
use crate::pointer::{PointerEvent, WheelEvent};

use super::{BackGestureUpdate, SwipeGesture, SwipeUpdate};

/// Tracks a trusted left-edge touch swipe or a trusted pixel-scroll swipe from a touchpad.
#[derive(Debug)]
pub struct BackGesture(SwipeGesture);

impl Default for BackGesture {
    fn default() -> Self {
        Self(SwipeGesture::back())
    }
}

impl BackGesture {
    /// Feed a dispatched UI event. Scale converts physical wheel deltas into logical pixels. Pointer positions in dispatched UI events are already logical.
    pub fn update(&mut self, event: &UiEvent, scale: f32) -> Option<BackGestureUpdate> {
        self.0.update(event, scale).map(back_update)
    }

    /// Feed a trusted phased pixel-scroll event, with the device scale used for its physical deltas.
    pub fn update_wheel(&mut self, wheel: WheelEvent, scale: f32) -> Option<BackGestureUpdate> {
        self.0.update_wheel(wheel, scale).map(back_update)
    }

    /// Feed a trusted touch event whose position is already in logical window coordinates.
    pub fn update_touch(&mut self, pointer: PointerEvent) -> Option<BackGestureUpdate> {
        self.0.update_touch(pointer).map(back_update)
    }

    /// The recognized displacement in logical pixels while the gesture is active.
    #[must_use]
    pub const fn offset(&self) -> Option<f32> {
        self.0.offset()
    }

    /// Discard the current sequence and return its cancellation update if it was recognized.
    pub fn cancel(&mut self) -> Option<BackGestureUpdate> {
        self.0.cancel().map(back_update)
    }
}

fn back_update(update: SwipeUpdate) -> BackGestureUpdate {
    BackGestureUpdate {
        source: update.source,
        phase: update.phase,
        offset: update.offset,
    }
}

#[cfg(test)]
mod tests {
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
}
