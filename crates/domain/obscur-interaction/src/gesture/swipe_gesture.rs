//! Directional swipe recognition for touchscreen contacts, native touchpad gestures and phased pixel scrolling.

use std::num::NonZeroUsize;

use obscur_geometry::Point;

use crate::event::UiEvent;
use crate::pointer::{
    PointerDevice, PointerEvent, PointerEventKind, PointerId, WheelDeltaMode, WheelEvent,
    WheelPhase,
};

use super::{
    GesturePhase, GestureSource, SwipeDirection, SwipeUpdate, TouchpadGestureDelta,
    TouchpadGestureEvent,
};

const MIN_DISTANCE: f32 = 72.0;
const AXIS_SLOP: f32 = 8.0;
const AXIS_RATIO: f32 = 1.75;
const TOUCH_EDGE: f32 = 32.0;

/// Recognizes a swipe and locks its direction until the sequence ends.
#[derive(Debug, Default)]
pub struct SwipeGesture {
    required_direction: Option<SwipeDirection>,
    left_edge_only: bool,
    direction: Option<SwipeDirection>,
    native_finger_count: Option<Option<NonZeroUsize>>,
    wheel_distance: Option<(f32, f32)>,
    required_finger_count: Option<NonZeroUsize>,
    contacts: Vec<(PointerId, Point)>,
    touch_origin: Option<Point>,
    offset: Option<f32>,
}

impl SwipeGesture {
    /// Restrict recognition to one direction.
    #[must_use]
    pub fn in_direction(direction: SwipeDirection) -> Self {
        Self {
            required_direction: Some(direction),
            ..Self::default()
        }
    }

    /// Requires an exact contact count from touchscreen or native touchpad input. With no filter, the recognizer accepts one-finger touchscreen swipes, native touchpad swipes, and phased pixel scroll.
    #[must_use]
    pub fn with_finger_count(mut self, count: NonZeroUsize) -> Self {
        self.cancel();
        self.required_finger_count = Some(count);
        self
    }

    pub(super) fn back() -> Self {
        Self {
            left_edge_only: true,
            ..Self::in_direction(SwipeDirection::Right)
        }
    }

    /// Feed a dispatched UI event. Scale converts physical wheel deltas into logical pixels. Pointer positions in dispatched UI events are already logical.
    pub fn update(&mut self, event: &UiEvent, scale: f32) -> Option<SwipeUpdate> {
        if let Some(gesture) = event.touchpad_gesture() {
            self.update_touchpad(gesture)
        } else if let Some(pointer) = event.pointer() {
            self.update_touch(pointer)
        } else {
            self.update_wheel(event.wheel()?, scale)
        }
    }

    /// Feed a trusted phased pixel-scroll event, with the device scale used for its physical deltas.
    pub fn update_wheel(&mut self, wheel: WheelEvent, scale: f32) -> Option<SwipeUpdate> {
        if self.native_finger_count.is_some()
            || !self.contacts.is_empty()
            || self.required_finger_count.is_some()
            || !wheel.trust.is_trusted()
            || wheel.mode != WheelDeltaMode::Pixels
        {
            return None;
        }
        if !scale.is_finite() || scale <= 0.0 {
            return self.cancel();
        }
        let delta_x = -wheel.delta_x / scale;
        let delta_y = if self.left_edge_only {
            wheel.delta_y.abs()
        } else {
            -wheel.delta_y
        } / scale;
        match wheel.phase {
            WheelPhase::Started => {
                let interrupted = self.cancel();
                if interrupted.is_some() {
                    self.wheel_distance = Some((delta_x, delta_y));
                    return interrupted;
                }
                self.wheel_distance = Some((0.0, 0.0));
            }
            WheelPhase::Moved => {
                self.wheel_distance.get_or_insert((0.0, 0.0));
            }
            WheelPhase::Ended => {}
            WheelPhase::Cancelled | WheelPhase::Unspecified => return self.cancel(),
        }
        let distance = self.wheel_distance.as_mut()?;
        distance.0 += delta_x;
        distance.1 += delta_y;
        let (horizontal, vertical) = *distance;
        self.sample(horizontal, vertical, wheel.phase == WheelPhase::Ended)
    }

    /// Feed a trusted touch event whose position is already in logical window coordinates.
    pub fn update_touch(&mut self, pointer: PointerEvent) -> Option<SwipeUpdate> {
        if pointer.device() != PointerDevice::Touch || !pointer.is_trusted() {
            return None;
        }
        let id = pointer.pointer();
        let position = pointer.position();
        if pointer.kind() == PointerEventKind::Down {
            let required = self
                .required_finger_count
                .unwrap_or(NonZeroUsize::MIN)
                .get();
            if self.contacts.iter().any(|(tracked, _)| *tracked == id) {
                return None;
            }
            if self.contacts.len() == required {
                return self.cancel();
            }
            let interrupted = if self.wheel_distance.is_some() {
                self.cancel()
            } else {
                None
            };
            if !self.left_edge_only || (0.0..=TOUCH_EDGE).contains(&position.x) {
                self.contacts.push((id, position));
                if self.contacts.len() == required {
                    self.touch_origin = Some(self.touch_center());
                }
            }
            return interrupted;
        }
        let (_, current) = self
            .contacts
            .iter_mut()
            .find(|(tracked, _)| *tracked == id)?;
        match pointer.kind() {
            PointerEventKind::Move | PointerEventKind::Up => {
                *current = position;
                let Some(origin) = self.touch_origin else {
                    return if pointer.kind() == PointerEventKind::Up {
                        self.cancel()
                    } else {
                        None
                    };
                };
                let center = self.touch_center();
                let horizontal = center.x - origin.x;
                let vertical = center.y - origin.y;
                if self.left_edge_only
                    && self.offset.is_none()
                    && (horizontal <= -AXIS_SLOP
                        || vertical.abs() >= AXIS_SLOP && vertical.abs() >= horizontal)
                {
                    return self.cancel();
                }
                self.sample(horizontal, vertical, pointer.kind() == PointerEventKind::Up)
            }
            PointerEventKind::Cancel => self.cancel(),
            _ => None,
        }
    }

    /// The displacement along the locked direction in logical pixels while the gesture is active.
    #[must_use]
    pub const fn offset(&self) -> Option<f32> {
        self.offset
    }

    /// Discard the current sequence and return its cancellation update if it was recognized.
    pub fn cancel(&mut self) -> Option<SwipeUpdate> {
        let (source, finger_count) = self.source_and_count();
        self.native_finger_count = None;
        self.contacts.clear();
        self.touch_origin = None;
        self.wheel_distance = None;
        let direction = self.direction.take();
        self.offset
            .take()
            .zip(direction)
            .map(|(offset, direction)| SwipeUpdate {
                source,
                phase: GesturePhase::Cancelled,
                direction,
                offset,
                finger_count,
            })
    }

    /// Feeds a native touchpad swipe with logical displacement and an optional contact count.
    pub fn update_touchpad(&mut self, event: TouchpadGestureEvent) -> Option<SwipeUpdate> {
        let TouchpadGestureDelta::Swipe(delta) = event.delta else {
            return None;
        };
        if !event.trust.is_trusted() || !self.contacts.is_empty() {
            return None;
        }
        if event.phase == GesturePhase::Started {
            let interrupted = self.cancel();
            if self.required_finger_count.is_none()
                || self.required_finger_count == event.finger_count
            {
                self.native_finger_count = Some(event.finger_count);
                self.wheel_distance = Some((0.0, 0.0));
            }
            if interrupted.is_some() {
                return interrupted;
            }
        }
        self.native_finger_count?;
        if event.phase == GesturePhase::Cancelled {
            return self.cancel();
        }
        let distance = self.wheel_distance.as_mut()?;
        distance.0 += delta.x;
        distance.1 += if self.left_edge_only {
            delta.y.abs()
        } else {
            delta.y
        };
        let (horizontal, vertical) = *distance;
        self.sample(horizontal, vertical, event.phase == GesturePhase::Completed)
    }

    fn source_and_count(&self) -> (GestureSource, Option<NonZeroUsize>) {
        if let Some(count) = self.native_finger_count {
            (GestureSource::Touchpad, count)
        } else if let Some(count) = NonZeroUsize::new(self.contacts.len()) {
            (GestureSource::Touchscreen, Some(count))
        } else {
            (GestureSource::PhasedScroll, None)
        }
    }

    fn touch_center(&self) -> Point {
        let (x, y) = self
            .contacts
            .iter()
            .fold((0.0, 0.0), |(x, y), (_, point)| (x + point.x, y + point.y));
        let count = self.contacts.len() as f32;
        Point::new(x / count, y / count)
    }

    fn sample(&mut self, horizontal: f32, vertical: f32, ended: bool) -> Option<SwipeUpdate> {
        let was_tracking = self.offset.is_some();
        if !horizontal.is_finite() || !vertical.is_finite() || ended && !was_tracking {
            return self.cancel();
        }
        let direction = match self.direction {
            Some(direction) => direction,
            None => {
                let candidate = if horizontal.abs() >= AXIS_SLOP
                    && horizontal.abs() >= AXIS_RATIO * vertical.abs()
                {
                    if horizontal > 0.0 {
                        SwipeDirection::Right
                    } else {
                        SwipeDirection::Left
                    }
                } else if vertical.abs() >= AXIS_SLOP
                    && vertical.abs() >= AXIS_RATIO * horizontal.abs()
                {
                    if vertical > 0.0 {
                        SwipeDirection::Down
                    } else {
                        SwipeDirection::Up
                    }
                } else {
                    return None;
                };
                if self
                    .required_direction
                    .is_some_and(|direction| direction != candidate)
                {
                    return None;
                }
                self.direction = Some(candidate);
                candidate
            }
        };
        let offset = match direction {
            SwipeDirection::Right => horizontal,
            SwipeDirection::Left => -horizontal,
            SwipeDirection::Down => vertical,
            SwipeDirection::Up => -vertical,
        }
        .max(0.0);
        self.offset = Some(offset);
        if ended {
            let mut update = self.cancel()?;
            if offset >= MIN_DISTANCE {
                update.phase = GesturePhase::Completed;
            }
            return Some(update);
        }
        let (source, finger_count) = self.source_and_count();
        Some(SwipeUpdate {
            source,
            phase: if was_tracking {
                GesturePhase::Updated
            } else {
                GesturePhase::Started
            },
            direction,
            offset,
            finger_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
        PointerEvent::new(
            PointerId::touch(1),
            PointerDevice::Touch,
            kind,
            Point::new(x, y),
        )
        .as_trusted()
    }

    #[test]
    fn every_direction_is_reported_with_positive_distance() {
        for (x, y, direction) in [
            (-1.0, 0.0, SwipeDirection::Left),
            (1.0, 0.0, SwipeDirection::Right),
            (0.0, -1.0, SwipeDirection::Up),
            (0.0, 1.0, SwipeDirection::Down),
        ] {
            let mut gesture = SwipeGesture::default();
            assert_eq!(
                gesture.update_touch(touch(PointerEventKind::Down, 100.0, 100.0)),
                None
            );
            for (kind, phase, offset) in [
                (PointerEventKind::Move, GesturePhase::Started, 20.0),
                (PointerEventKind::Move, GesturePhase::Updated, 40.0),
                (PointerEventKind::Up, GesturePhase::Completed, 90.0),
            ] {
                assert_eq!(
                    gesture.update_touch(touch(kind, 100.0 + x * offset, 100.0 + y * offset)),
                    Some(SwipeUpdate {
                        direction,
                        phase,
                        offset,
                        source: GestureSource::Touchscreen,
                        finger_count: Some(NonZeroUsize::MIN),
                    }),
                );
            }
        }
    }

    #[test]
    fn two_finger_swipe_tracks_the_centroid_and_reports_count_through_completion() {
        let count = NonZeroUsize::new(2).unwrap();
        let mut gesture = SwipeGesture::default().with_finger_count(count);
        let event = |id, kind, x| {
            PointerEvent::new(
                PointerId::touch(id),
                PointerDevice::Touch,
                kind,
                Point::new(x, 0.0),
            )
            .as_trusted()
        };
        assert_eq!(
            gesture.update_touch(event(1, PointerEventKind::Down, 0.0)),
            None
        );
        assert_eq!(
            gesture.update_touch(event(1, PointerEventKind::Move, 20.0)),
            None
        );
        assert_eq!(
            gesture.update_touch(event(2, PointerEventKind::Down, 40.0)),
            None
        );
        for (id, kind, x, phase, offset) in [
            (1, PointerEventKind::Move, 60.0, GesturePhase::Started, 20.0),
            (
                2,
                PointerEventKind::Move,
                130.0,
                GesturePhase::Updated,
                65.0,
            ),
            (1, PointerEventKind::Up, 80.0, GesturePhase::Completed, 75.0),
        ] {
            assert_eq!(
                gesture.update_touch(event(id, kind, x)),
                Some(SwipeUpdate {
                    direction: SwipeDirection::Right,
                    phase,
                    offset,
                    source: GestureSource::Touchscreen,
                    finger_count: Some(count),
                })
            );
        }
    }

    #[test]
    fn a_vertical_pixel_scroll_reports_logical_finger_distance() {
        let mut gesture = SwipeGesture::in_direction(SwipeDirection::Down);
        let wheel = WheelEvent::new(Point::new(0.0, 0.0), -40.0)
            .with_phase(WheelPhase::Started)
            .as_trusted();
        assert_eq!(
            gesture.update_wheel(wheel, 2.0),
            Some(SwipeUpdate {
                direction: SwipeDirection::Down,
                phase: GesturePhase::Started,
                offset: 20.0,
                source: GestureSource::PhasedScroll,
                finger_count: None,
            }),
        );
        assert_eq!(
            gesture.cancel().map(|event| event.phase),
            Some(GesturePhase::Cancelled)
        );
    }
}
