//! The typed payload a [`UiEvent`](super::UiEvent) carries to a handler
//! (spec §2.3, §2.4).

use crate::command::KeyEvent;
use crate::pointer::PointerEvent;

/// The typed input payload carried by a [`UiEvent`](super::UiEvent) (spec §2.3,
/// §2.4).
///
/// §2.3 closes with a sentence that rules out the shape a payload is *usually*
/// smuggled in with: *"Events are typed. String-based method dispatch is not
/// used."* So the payload is an enum of the concrete device events the runtime
/// already models — not a property bag, not a `dyn Any`, and not a name a
/// handler looks up. A handler reads it by matching, and the compiler knows which
/// fields exist.
///
/// # Why the envelope does not simply *contain* the pointer fields
///
/// §2.4 lists nine facts a pointer event must carry (pointer ID, device type,
/// buttons, logical position and delta, pressure, tilt, rotation,
/// primary-pointer status, hover capability); a keyboard event carries none of
/// them and carries a physical/logical key pair instead. Flattening both into one
/// struct would give every handler a set of fields that are meaningless for the
/// event it actually received — which is the untyped shape §2.3 excludes. The
/// variants keep "which facts exist" a property of the event's *kind*.
///
/// # `Empty`
///
/// [`Empty`](Self::Empty) is not a device: it is the honest payload for a
/// command that no device produced — an accessibility action, a menu item, a
/// programmatic dispatch. Such a command still propagates and still reaches
/// handlers, so it still needs an envelope; it simply has no device state to
/// report, and a handler that matches on the variant sees that rather than
/// reading a zeroed position that never existed.
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
