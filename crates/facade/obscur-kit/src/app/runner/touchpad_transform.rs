use std::num::NonZeroUsize;

use obscur_geometry::Point;
use obscur_interaction::event::EventTrust;
use obscur_interaction::gesture::{GesturePhase, TouchpadGestureDelta, TouchpadGestureEvent};
use winit::event::TouchPhase;

use super::transform_component::TransformComponent;

#[derive(Default)]
pub(super) struct TouchpadTransform {
    channels: [bool; 2],
    current: Option<TouchpadGestureEvent>,
}

impl TouchpadTransform {
    pub(super) fn update(
        &mut self,
        component: TransformComponent,
        phase: TouchPhase,
        position: Point,
        timestamp_micros: u64,
    ) -> Option<TouchpadGestureEvent> {
        let index = match component {
            TransformComponent::Magnification(_) => 0,
            TransformComponent::Rotation(_) => 1,
        };
        let started = self.current.is_none();
        match phase {
            TouchPhase::Started => self.channels[index] = true,
            TouchPhase::Moved if !self.channels[index] => return None,
            TouchPhase::Ended if self.channels[index] => self.channels[index] = false,
            TouchPhase::Cancelled if self.channels[index] => {
                self.channels = [false; 2];
                let mut event = self.current.take()?;
                event.phase = GesturePhase::Cancelled;
                event.timestamp_micros = timestamp_micros;
                return Some(event);
            }
            TouchPhase::Moved => {}
            TouchPhase::Ended | TouchPhase::Cancelled => return None,
        }

        let mut event = self.current.unwrap_or(TouchpadGestureEvent {
            phase: GesturePhase::Started,
            position,
            finger_count: NonZeroUsize::new(2),
            delta: TouchpadGestureDelta::Transform {
                scale: 1.0,
                rotation: 0.0,
            },
            trust: EventTrust::Trusted,
            timestamp_micros,
        });
        let TouchpadGestureDelta::Transform {
            mut scale,
            mut rotation,
        } = event.delta
        else {
            return None;
        };
        match component {
            TransformComponent::Magnification(delta) => scale *= 1.0 + delta,
            TransformComponent::Rotation(delta) => rotation += delta,
        }
        if !scale.is_finite() || scale <= 0.0 || !rotation.is_finite() {
            self.channels = [false; 2];
            let mut event = self.current.take()?;
            event.phase = GesturePhase::Cancelled;
            event.timestamp_micros = timestamp_micros;
            return Some(event);
        }
        event.delta = TouchpadGestureDelta::Transform { scale, rotation };
        event.timestamp_micros = timestamp_micros;
        event.phase = if started {
            GesturePhase::Started
        } else if self.channels.iter().any(|active| *active) {
            GesturePhase::Updated
        } else {
            GesturePhase::Completed
        };
        self.current = (event.phase != GesturePhase::Completed).then_some(event);
        Some(event)
    }
}
