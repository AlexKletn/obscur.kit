//! The typed payload a [`UiEvent`](super::UiEvent) carries to a handler
//! (spec §2.3, §2.4).

use crate::command::KeyEvent;
use crate::gamepad::GamepadEvent;
use crate::gesture::{SystemBackEvent, TouchpadGestureEvent};
use crate::pointer::{PointerEvent, WheelEvent};

#[doc = include_str!("../../docs/api/event/event_payload.doc/EventPayload.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EventPayload {
    /// No device payload: the event was produced by something other than an
    /// input device (a menu, assistive technology, a programmatic dispatch).
    #[default]
    Empty,
    /// A pointer event's §2.4 payload.
    Pointer(PointerEvent),
    /// A keyboard key transition's payload (spec §2.6 physical key / logical
    /// key).
    Key(KeyEvent),
    /// Native touchpad gesture sample.
    TouchpadGesture(TouchpadGestureEvent),
    #[doc = include_str!("../../docs/api/event/event_payload.doc/Wheel.md")]
    Wheel(WheelEvent),
    /// Predictive Back data supplied by the window host.
    SystemBack(SystemBackEvent),
    /// Input from a gamepad adapter.
    Gamepad(GamepadEvent),
}

impl EventPayload {
    /// Returns the pointer payload, or `None` for any other kind of event.
    #[must_use]
    pub const fn pointer(&self) -> Option<PointerEvent> {
        match self {
            Self::Pointer(pointer) => Some(*pointer),
            _ => None,
        }
    }

    /// Returns the keyboard payload, or `None` for any other kind of event.
    #[must_use]
    pub const fn key(&self) -> Option<KeyEvent> {
        match self {
            Self::Key(key) => Some(*key),
            _ => None,
        }
    }

    /// Returns the wheel payload, or `None` for any other kind of event.
    #[must_use]
    pub const fn wheel(&self) -> Option<WheelEvent> {
        match self {
            Self::Wheel(wheel) => Some(*wheel),
            _ => None,
        }
    }

    /// Returns whether this event carries no device payload.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[cfg(test)]
mod tests {
    use obscur_geometry::Point;

    use super::*;
    use crate::command::{LogicalKey, PhysicalKey};
    use crate::pointer::{PointerDevice, PointerEventKind, PointerId};

    fn wheel() -> WheelEvent {
        WheelEvent::new(Point::new(5.0, 6.0), 40.0)
    }

    fn pointer() -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            Point::new(1.0, 2.0),
        )
    }

    fn key() -> KeyEvent {
        KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a')))
    }

    #[test]
    fn default_is_empty() {
        assert!(EventPayload::default().is_empty());
        assert_eq!(EventPayload::default().pointer(), None);
        assert_eq!(EventPayload::default().key(), None);
        assert_eq!(EventPayload::default().wheel(), None);
    }

    /// The accessors are narrowing, not coercing: a keyboard payload never reads
    /// out as a pointer at (0, 0), which is the failure a flattened envelope
    /// would produce silently.
    #[test]
    fn each_accessor_answers_only_for_its_own_variant() {
        let p = EventPayload::Pointer(pointer());
        assert_eq!(p.pointer(), Some(pointer()));
        assert_eq!(p.key(), None);
        assert!(!p.is_empty());

        let k = EventPayload::Key(key());
        assert_eq!(k.key(), Some(key()));
        assert_eq!(k.pointer(), None);
        assert!(!k.is_empty());

        // The wheel is the case the narrowing rule was most at risk of losing: a
        // notch reused the pointer variant, it would read out as a pointer that
        // moved to the wheel's position — a move that never happened, and one
        // the model could not tell from a real one.
        let w = EventPayload::Wheel(wheel());
        assert_eq!(w.wheel(), Some(wheel()));
        assert_eq!(w.pointer(), None);
        assert_eq!(w.key(), None);
        assert!(!w.is_empty());
        assert_eq!(EventPayload::Pointer(pointer()).wheel(), None);
        assert_eq!(EventPayload::Key(key()).wheel(), None);
    }
}
