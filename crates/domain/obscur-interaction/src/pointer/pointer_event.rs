//! The [`PointerEvent`] payload carried by a hit-tested pointer event and its
//! [`PointerEventKind`] discriminant (spec §4.7).

use obscur_geometry::Point;

use super::{PointerButtons, PointerDevice, PointerId};

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
}

impl PointerEvent {
    /// Creates a pointer event with no buttons held, zero pressure, and a zero
    /// timestamp.
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
        }
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
