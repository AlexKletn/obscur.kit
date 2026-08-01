#![doc = include_str!("event_payload.doc/_module.md")]

use crate::command::KeyEvent;
use crate::pointer::PointerEvent;

#[doc = include_str!("event_payload.doc/EventPayload.md")]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EventPayload {
    #[doc = include_str!("event_payload.doc/Empty.md")]
    #[default]
    Empty,
    #[doc = include_str!("event_payload.doc/Pointer.md")]
    Pointer(PointerEvent),
    #[doc = include_str!("event_payload.doc/Key.md")]
    Key(KeyEvent),
}

impl EventPayload {
    #[doc = include_str!("event_payload.doc/pointer.md")]
    #[must_use]
    pub const fn pointer(&self) -> Option<PointerEvent> {
        match self {
            Self::Pointer(pointer) => Some(*pointer),
            _ => None,
        }
    }

    #[doc = include_str!("event_payload.doc/key.md")]
    #[must_use]
    pub const fn key(&self) -> Option<KeyEvent> {
        match self {
            Self::Key(key) => Some(*key),
            _ => None,
        }
    }

    #[doc = include_str!("event_payload.doc/is_empty.md")]
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
    }
}
