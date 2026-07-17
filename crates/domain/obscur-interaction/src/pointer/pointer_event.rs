//! The [`PointerEvent`] payload carried by a hit-tested pointer event and its
//! [`PointerEventKind`] discriminant (spec §4.7).

use obscur_geometry::Point;

use super::{PointerButtons, PointerDevice, PointerId};
use crate::event::EventTrust;

/// The lifecycle kind of a [`PointerEvent`] (spec §4.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerEventKind {
    /// A button was pressed or a contact began.
    Down,
    /// A button was released or a contact ended.
    Up,
    /// The pointer moved within the target.
    Move,
    /// The pointer entered the target.
    Enter,
    /// The pointer left the target.
    Leave,
    /// The interaction was cancelled by the system (for example, capture lost).
    Cancel,
}

impl PointerEventKind {
    /// Returns whether this kind terminates the pointer's interaction with the
    /// target ([`Up`](Self::Up), [`Cancel`](Self::Cancel), or
    /// [`Leave`](Self::Leave)).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Up | Self::Cancel | Self::Leave)
    }
}

/// A single pointer event delivered to a hit-tested target (spec §4.7).
///
/// Each event describes one pointer's state at one instant: which
/// [`pointer`](Self::pointer) and [`device`](Self::device) produced it, the
/// [`kind`](Self::kind) of change, the [`position`](Self::position) in the
/// target's coordinate space, the [`buttons`](Self::buttons) held *after* this
/// event, the contact [`pressure`](Self::pressure) (`0.0` for hover or a
/// device without pressure), and a logical [`timestamp`](Self::timestamp)
/// supplied by the caller — this crate never reads a clock.
///
/// It also carries §2.3's trusted/untrusted origin ([`trust`](Self::trust)).
/// [`new`](Self::new) produces an [`Untrusted`](EventTrust::Untrusted) event:
/// trust is conferred at the platform boundary by [`as_trusted`](Self::as_trusted)
/// and is never assumed. See [`EventTrust`] for why that direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    /// The pointer that produced this event.
    pub pointer: PointerId,
    /// The device that produced this event.
    pub device: PointerDevice,
    /// The lifecycle kind of this event.
    pub kind: PointerEventKind,
    /// The pointer position in the hit target's coordinate space.
    pub position: Point,
    /// The buttons held down after this event.
    pub buttons: PointerButtons,
    /// The contact pressure in `0.0..=1.0` (`0.0` for hover / no contact).
    pub pressure: f32,
    /// A logical timestamp in microseconds, supplied by the caller.
    pub timestamp_micros: u64,
    /// Whether this came from a real user gesture (§2.3 trusted/untrusted
    /// origin). Untrusted unless a platform adapter vouched for it.
    pub trust: EventTrust,
}

impl PointerEvent {
    /// Creates a pointer event with no buttons held, zero pressure, a zero
    /// timestamp, and an [`Untrusted`](EventTrust::Untrusted) origin.
    ///
    /// Untrusted is deliberate, not a placeholder: a synthetic event that forgets
    /// to say what it is must not pass for a real gesture. The one producer of
    /// real input — the platform adapter holding the OS event — opts in with
    /// [`as_trusted`](Self::as_trusted).
    #[must_use]
    pub const fn new(
        pointer: PointerId,
        device: PointerDevice,
        kind: PointerEventKind,
        position: Point,
    ) -> Self {
        Self {
            pointer,
            device,
            kind,
            position,
            buttons: PointerButtons::new(),
            pressure: 0.0,
            timestamp_micros: 0,
            trust: EventTrust::Untrusted,
        }
    }

    /// Returns this event marked as a real user gesture (fluent).
    ///
    /// **Only a platform adapter converting an actual OS input event may call
    /// this.** It is the single point at which trust enters the runtime; every
    /// other producer (DevTools `Inject` per §7.6, the §7.4 headless harness)
    /// must leave the event untrusted.
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    /// Returns this event's trusted/untrusted origin (§2.3).
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    /// Returns whether this event came from a real user gesture (§2.3).
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
    }

    /// Returns this event with its held buttons set (fluent).
    #[must_use]
    pub const fn with_buttons(mut self, buttons: PointerButtons) -> Self {
        self.buttons = buttons;
        self
    }

    /// Returns this event with its contact pressure set, clamped to
    /// `0.0..=1.0` (fluent).
    #[must_use]
    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = pressure.clamp(0.0, 1.0);
        self
    }

    /// Returns this event with its logical timestamp set (fluent).
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    /// Returns the pointer that produced this event.
    #[must_use]
    pub const fn pointer(&self) -> PointerId {
        self.pointer
    }

    /// Returns the device that produced this event.
    #[must_use]
    pub const fn device(&self) -> PointerDevice {
        self.device
    }

    /// Returns the lifecycle kind of this event.
    #[must_use]
    pub const fn kind(&self) -> PointerEventKind {
        self.kind
    }

    /// Returns the pointer position in the hit target's coordinate space.
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }

    /// Returns the buttons held down after this event.
    #[must_use]
    pub const fn buttons(&self) -> PointerButtons {
        self.buttons
    }

    /// Returns the contact pressure in `0.0..=1.0`.
    #[must_use]
    pub const fn pressure(&self) -> f32 {
        self.pressure
    }

    /// Returns the logical timestamp, in microseconds, supplied by the caller.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    /// Returns whether any button is pressed after this event.
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        !self.buttons.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer::PointerButton;

    fn base() -> PointerEvent {
        PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            PointerEventKind::Down,
            Point::new(3.0, 4.0),
        )
    }

    #[test]
    fn new_defaults() {
        let e = base();
        assert_eq!(e.pointer(), PointerId::MOUSE);
        assert_eq!(e.device(), PointerDevice::Mouse);
        assert_eq!(e.kind(), PointerEventKind::Down);
        assert_eq!(e.position(), Point::new(3.0, 4.0));
        assert!(e.buttons().is_empty());
        assert_eq!(e.pressure(), 0.0);
        assert_eq!(e.timestamp(), 0);
        assert!(!e.is_pressed());
        // §2.3: a bare event is NOT a user gesture. This default is the whole
        // security argument — see `EventTrust`.
        assert_eq!(e.trust(), EventTrust::Untrusted);
        assert!(!e.is_trusted());
    }

    /// §2.3: trust is conferred, and conferring it changes nothing else — a
    /// trusted event is the same event, differently labelled.
    #[test]
    fn as_trusted_marks_the_event_and_touches_nothing_else() {
        let plain = base();
        let trusted = base().as_trusted();

        assert!(!plain.is_trusted());
        assert!(trusted.is_trusted());
        assert_eq!(trusted.trust(), EventTrust::Trusted);
        assert_ne!(
            plain.trust(),
            trusted.trust(),
            "the two must be distinguishable"
        );

        assert_eq!(trusted.position(), plain.position());
        assert_eq!(trusted.kind(), plain.kind());
        assert_eq!(trusted.pointer(), plain.pointer());
    }

    #[test]
    fn builder_sets_fields() {
        let mut buttons = PointerButtons::new();
        buttons.press(PointerButton::Primary);
        let e = base()
            .with_buttons(buttons)
            .with_pressure(0.5)
            .with_timestamp(1_000);
        assert!(e.is_pressed());
        assert!(e.buttons().contains(PointerButton::Primary));
        assert_eq!(e.pressure(), 0.5);
        assert_eq!(e.timestamp(), 1_000);
    }

    #[test]
    fn pressure_is_clamped() {
        assert_eq!(base().with_pressure(2.0).pressure(), 1.0);
        assert_eq!(base().with_pressure(-1.0).pressure(), 0.0);
    }

    #[test]
    fn kind_is_terminal() {
        assert!(PointerEventKind::Up.is_terminal());
        assert!(PointerEventKind::Cancel.is_terminal());
        assert!(PointerEventKind::Leave.is_terminal());
        assert!(!PointerEventKind::Down.is_terminal());
        assert!(!PointerEventKind::Move.is_terminal());
        assert!(!PointerEventKind::Enter.is_terminal());
    }
}
