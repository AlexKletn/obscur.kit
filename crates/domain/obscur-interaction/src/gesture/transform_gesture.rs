//! A combined pinch and rotate recognizer.

use std::f32::consts::{PI, TAU};
use std::num::NonZeroUsize;

use obscur_geometry::Point;

use crate::event::UiEvent;
use crate::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};

use super::{
    GesturePhase, GestureSource, TouchpadGestureDelta, TouchpadGestureEvent, TransformGestureUpdate,
};

/// Tracks pinch and rotation from two trusted touchscreen contacts or a native touchpad gesture.
#[derive(Debug, Default)]
pub struct TransformGesture {
    contacts: [Option<(PointerId, Point)>; 2],
    initial_distance: Option<f32>,
    previous_angle: f32,
    rotation: f32,
    active: Option<TransformGestureUpdate>,
}

impl TransformGesture {
    /// Feed a dispatched UI event carrying touchscreen pointer data or native touchpad measurements.
    pub fn update(&mut self, event: &UiEvent) -> Option<TransformGestureUpdate> {
        if let Some(gesture) = event.touchpad_gesture() {
            self.update_touchpad(gesture)
        } else {
            self.update_touch(event.pointer()?)
        }
    }

    /// Feed a trusted touch event in logical window coordinates.
    pub fn update_touch(&mut self, pointer: PointerEvent) -> Option<TransformGestureUpdate> {
        if self
            .active
            .is_some_and(|update| update.source == GestureSource::Touchpad)
            || pointer.device() != PointerDevice::Touch
            || !pointer.is_trusted()
        {
            return None;
        }
        let id = pointer.pointer();
        let index = self
            .contacts
            .iter()
            .position(|entry| entry.is_some_and(|(tracked, _)| tracked == id));
        let position = pointer.position();
        if !position.x.is_finite() || !position.y.is_finite() {
            return self.cancel();
        }
        match pointer.kind() {
            PointerEventKind::Down => {
                if index.is_some() {
                    return None;
                }
                let Some(slot) = self.contacts.iter_mut().find(|slot| slot.is_none()) else {
                    return self.cancel();
                };
                *slot = Some((id, position));
                self.sample()
            }
            PointerEventKind::Move => {
                self.contacts[index?] = Some((id, position));
                self.sample()
            }
            PointerEventKind::Up => {
                let index = index?;
                if self.active.is_some() {
                    self.contacts[index] = Some((id, position));
                    self.sample();
                }
                self.contacts[index] = None;
                self.reset_transform();
                self.active.take().map(|update| TransformGestureUpdate {
                    phase: GesturePhase::Completed,
                    ..update
                })
            }
            PointerEventKind::Cancel if index.is_some() => self.cancel(),
            _ => None,
        }
    }

    /// End the current sequence without completing it.
    pub fn cancel(&mut self) -> Option<TransformGestureUpdate> {
        self.contacts = [None; 2];
        self.reset_transform();
        self.active.take().map(|update| TransformGestureUpdate {
            phase: GesturePhase::Cancelled,
            ..update
        })
    }

    /// Feeds scale and rotation from a native touchpad sequence.
    pub fn update_touchpad(
        &mut self,
        event: TouchpadGestureEvent,
    ) -> Option<TransformGestureUpdate> {
        let TouchpadGestureDelta::Transform { scale, rotation } = event.delta else {
            return None;
        };
        if !event.trust.is_trusted() || self.contacts.iter().any(Option::is_some) {
            return None;
        }
        if event.phase == GesturePhase::Cancelled
            || !scale.is_finite()
            || scale <= 0.0
            || !rotation.is_finite()
        {
            return self.cancel();
        }
        let (center, finger_count) = if event.phase == GesturePhase::Started {
            (event.position, event.finger_count)
        } else {
            let active = self.active?;
            (active.center, active.finger_count)
        };
        let update = TransformGestureUpdate {
            source: GestureSource::Touchpad,
            phase: event.phase,
            center,
            finger_count,
            scale,
            rotation,
        };
        self.active = (event.phase != GesturePhase::Completed).then_some(update);
        Some(update)
    }

    fn reset_transform(&mut self) {
        self.initial_distance = None;
        self.rotation = 0.0;
    }

    fn sample(&mut self) -> Option<TransformGestureUpdate> {
        let [Some((_, a)), Some((_, b))] = self.contacts else {
            return None;
        };
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let distance = dx.hypot(dy);
        if !distance.is_finite() || distance < 1.0 {
            return None;
        }
        let angle = dy.atan2(dx);
        let Some(initial) = self.initial_distance else {
            self.initial_distance = Some(distance);
            self.previous_angle = angle;
            return None;
        };
        self.rotation += (angle - self.previous_angle + PI).rem_euclid(TAU) - PI;
        self.previous_angle = angle;
        let scale = distance / initial;
        let active = self.active.is_some();
        if !active && (scale - 1.0).abs() < 0.02 && self.rotation.abs() < 0.035 {
            return None;
        }
        let update = TransformGestureUpdate {
            source: GestureSource::Touchscreen,
            finger_count: NonZeroUsize::new(2),
            phase: if active {
                GesturePhase::Updated
            } else {
                GesturePhase::Started
            },
            center: Point::new(a.x + dx * 0.5, a.y + dy * 0.5),
            scale,
            rotation: self.rotation,
        };
        self.active = Some(update);
        Some(update)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(id: u64, kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
        PointerEvent::new(
            PointerId::touch(id),
            PointerDevice::Touch,
            kind,
            Point::new(x, y),
        )
        .as_trusted()
    }

    #[test]
    fn two_contacts_report_pinch_center_and_clockwise_rotation() {
        let mut gesture = TransformGesture::default();
        assert_eq!(
            gesture.update_touch(touch(1, PointerEventKind::Down, 100.0, 100.0)),
            None
        );
        assert_eq!(
            gesture.update_touch(touch(2, PointerEventKind::Down, 200.0, 100.0)),
            None
        );
        let pinch = gesture
            .update_touch(touch(2, PointerEventKind::Move, 250.0, 100.0))
            .unwrap();
        assert_eq!(pinch.phase, GesturePhase::Started);
        assert_eq!(pinch.center, Point::new(175.0, 100.0));
        assert_eq!(pinch.scale, 1.5);
        assert_eq!(pinch.rotation, 0.0);

        let rotation = gesture
            .update_touch(touch(2, PointerEventKind::Move, 100.0, 250.0))
            .unwrap();
        assert_eq!(rotation.phase, GesturePhase::Updated);
        assert!((rotation.rotation - PI / 2.0).abs() < 0.0001);
        assert_eq!(rotation.scale, 1.5);
        assert_eq!(
            gesture
                .update_touch(touch(2, PointerEventKind::Up, 100.0, 250.0))
                .unwrap()
                .phase,
            GesturePhase::Completed
        );
        assert_eq!(
            gesture.update_touch(touch(1, PointerEventKind::Up, 100.0, 100.0)),
            None
        );
    }

    #[test]
    fn a_third_contact_cancels_an_active_transform_once() {
        let mut gesture = TransformGesture::default();
        gesture.update_touch(touch(1, PointerEventKind::Down, 30.0, 30.0));
        gesture.update_touch(touch(2, PointerEventKind::Down, 70.0, 30.0));
        gesture.update_touch(touch(2, PointerEventKind::Move, 90.0, 30.0));
        assert_eq!(
            gesture
                .update_touch(touch(3, PointerEventKind::Down, 110.0, 30.0))
                .unwrap()
                .phase,
            GesturePhase::Cancelled
        );
        assert_eq!(
            gesture.update_touch(touch(2, PointerEventKind::Up, 90.0, 30.0)),
            None
        );
    }
}
