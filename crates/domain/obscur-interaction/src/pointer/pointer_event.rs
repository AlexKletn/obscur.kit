#![doc = include_str!("pointer_event.doc/_module.md")]

use obscur_geometry::Point;

use super::{PointerButton, PointerButtons, PointerDelta, PointerDevice, PointerId, PointerTilt};
use crate::event::EventTrust;

#[doc = include_str!("pointer_event.doc/PointerEventKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerEventKind {
    #[doc = include_str!("pointer_event.doc/Down.md")]
    Down,
    #[doc = include_str!("pointer_event.doc/Up.md")]
    Up,
    #[doc = include_str!("pointer_event.doc/Move.md")]
    Move,
    #[doc = include_str!("pointer_event.doc/Enter.md")]
    Enter,
    #[doc = include_str!("pointer_event.doc/Leave.md")]
    Leave,
    #[doc = include_str!("pointer_event.doc/Cancel.md")]
    Cancel,
}

impl PointerEventKind {
    #[doc = include_str!("pointer_event.doc/is_terminal.md")]
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Up | Self::Cancel | Self::Leave)
    }
}

#[doc = include_str!("pointer_event.doc/PointerEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    #[doc = include_str!("pointer_event.doc/pointer.md")]
    pub pointer: PointerId,
    #[doc = include_str!("pointer_event.doc/device.md")]
    pub device: PointerDevice,
    #[doc = include_str!("pointer_event.doc/kind.md")]
    pub kind: PointerEventKind,
    #[doc = include_str!("pointer_event.doc/position.md")]
    pub position: Point,
    #[doc = include_str!("pointer_event.doc/delta.md")]
    pub delta: PointerDelta,
    #[doc = include_str!("pointer_event.doc/buttons.md")]
    pub buttons: PointerButtons,
    #[doc = include_str!("pointer_event.doc/button.md")]
    pub button: Option<PointerButton>,
    #[doc = include_str!("pointer_event.doc/pressure.md")]
    pub pressure: f32,
    #[doc = include_str!("pointer_event.doc/tilt.md")]
    pub tilt: PointerTilt,
    #[doc = include_str!("pointer_event.doc/rotation.md")]
    pub rotation: f32,
    #[doc = include_str!("pointer_event.doc/primary.md")]
    pub primary: bool,
    #[doc = include_str!("pointer_event.doc/hover_capable.md")]
    pub hover_capable: bool,
    #[doc = include_str!("pointer_event.doc/timestamp_micros.md")]
    pub timestamp_micros: u64,
    #[doc = include_str!("pointer_event.doc/trust.md")]
    pub trust: EventTrust,
}

impl PointerEvent {
    #[doc = include_str!("pointer_event.doc/new.md")]
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
            delta: PointerDelta::ZERO,
            buttons: PointerButtons::new(),
            button: None,
            pressure: 0.0,
            tilt: PointerTilt::UPRIGHT,
            rotation: 0.0,
            primary: true,
            hover_capable: device.hovers_by_default(),
            timestamp_micros: 0,
            trust: EventTrust::Untrusted,
        }
    }

    #[doc = include_str!("pointer_event.doc/as_trusted.md")]
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    #[doc = include_str!("pointer_event.doc/trust_2.md")]
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    #[doc = include_str!("pointer_event.doc/is_trusted.md")]
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
    }

    #[doc = include_str!("pointer_event.doc/with_buttons.md")]
    #[must_use]
    pub const fn with_buttons(mut self, buttons: PointerButtons) -> Self {
        self.buttons = buttons;
        self
    }

    #[doc = include_str!("pointer_event.doc/with_button.md")]
    #[must_use]
    pub const fn with_button(mut self, button: PointerButton) -> Self {
        self.button = Some(button);
        self
    }

    #[doc = include_str!("pointer_event.doc/with_pressure.md")]
    #[must_use]
    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = pressure.clamp(0.0, 1.0);
        self
    }

    #[doc = include_str!("pointer_event.doc/with_delta.md")]
    #[must_use]
    pub const fn with_delta(mut self, delta: PointerDelta) -> Self {
        self.delta = delta;
        self
    }

    #[doc = include_str!("pointer_event.doc/with_tilt.md")]
    #[must_use]
    pub const fn with_tilt(mut self, tilt: PointerTilt) -> Self {
        self.tilt = tilt;
        self
    }

    #[doc = include_str!("pointer_event.doc/with_rotation.md")]
    #[must_use]
    pub fn with_rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees.rem_euclid(360.0);
        self
    }

    #[doc = include_str!("pointer_event.doc/with_primary.md")]
    #[must_use]
    pub const fn with_primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    #[doc = include_str!("pointer_event.doc/with_hover_capability.md")]
    #[must_use]
    pub const fn with_hover_capability(mut self, hover_capable: bool) -> Self {
        self.hover_capable = hover_capable;
        self
    }

    #[doc = include_str!("pointer_event.doc/with_timestamp.md")]
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    #[doc = include_str!("pointer_event.doc/pointer_2.md")]
    #[must_use]
    pub const fn pointer(&self) -> PointerId {
        self.pointer
    }

    #[doc = include_str!("pointer_event.doc/device_2.md")]
    #[must_use]
    pub const fn device(&self) -> PointerDevice {
        self.device
    }

    #[doc = include_str!("pointer_event.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> PointerEventKind {
        self.kind
    }

    #[doc = include_str!("pointer_event.doc/position_2.md")]
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }

    #[doc = include_str!("pointer_event.doc/buttons_2.md")]
    #[must_use]
    pub const fn buttons(&self) -> PointerButtons {
        self.buttons
    }

    #[doc = include_str!("pointer_event.doc/button_2.md")]
    #[must_use]
    pub const fn button(&self) -> Option<PointerButton> {
        self.button
    }

    #[doc = include_str!("pointer_event.doc/pressure_2.md")]
    #[must_use]
    pub const fn pressure(&self) -> f32 {
        self.pressure
    }

    #[doc = include_str!("pointer_event.doc/delta_2.md")]
    #[must_use]
    pub const fn delta(&self) -> PointerDelta {
        self.delta
    }

    #[doc = include_str!("pointer_event.doc/tilt_2.md")]
    #[must_use]
    pub const fn tilt(&self) -> PointerTilt {
        self.tilt
    }

    #[doc = include_str!("pointer_event.doc/rotation_2.md")]
    #[must_use]
    pub const fn rotation(&self) -> f32 {
        self.rotation
    }

    #[doc = include_str!("pointer_event.doc/is_primary.md")]
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.primary
    }

    #[doc = include_str!("pointer_event.doc/hover_capable_2.md")]
    #[must_use]
    pub const fn hover_capable(&self) -> bool {
        self.hover_capable
    }

    #[doc = include_str!("pointer_event.doc/timestamp.md")]
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    #[doc = include_str!("pointer_event.doc/is_pressed.md")]
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
        assert_eq!(
            e.button(),
            None,
            "a producer that does not name the moved button must say nothing \
             rather than claim the primary one"
        );
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

    /// The held set and the moved button answer two different questions, and a
    /// chord is where the difference stops being academic: with the primary
    /// already down, pressing the secondary reports both as held while naming
    /// only the one that moved. Reading "which button" off the set would call
    /// this press a primary one.
    #[test]
    fn the_moved_button_is_not_recoverable_from_the_held_set() {
        let mut chord = PointerButtons::new();
        chord.press(PointerButton::Primary);
        chord.press(PointerButton::Secondary);

        let secondary_joined = base()
            .with_buttons(chord)
            .with_button(PointerButton::Secondary);
        let primary_joined = base()
            .with_buttons(chord)
            .with_button(PointerButton::Primary);

        assert_eq!(secondary_joined.buttons(), primary_joined.buttons());
        assert_eq!(secondary_joined.button(), Some(PointerButton::Secondary));
        assert_eq!(primary_joined.button(), Some(PointerButton::Primary));
        assert_ne!(secondary_joined.button(), primary_joined.button());
    }

    #[test]
    fn pressure_is_clamped() {
        assert_eq!(base().with_pressure(2.0).pressure(), 1.0);
        assert_eq!(base().with_pressure(-1.0).pressure(), 0.0);
    }

    /// §2.4's four previously-absent facts: a producer that knows them can say
    /// so, and one that does not reports "nothing to report" rather than a
    /// plausible lie.
    #[test]
    fn the_remaining_2_4_fields_default_to_nothing_reported_and_are_settable() {
        let bare = base();
        assert_eq!(bare.delta(), PointerDelta::ZERO, "a press moved nothing");
        assert_eq!(bare.tilt(), PointerTilt::UPRIGHT, "a mouse has no barrel");
        assert_eq!(bare.rotation(), 0.0, "and no twist");
        assert!(bare.is_primary(), "a lone pointer is the primary one");

        let pen = PointerEvent::new(
            PointerId::from_raw(3),
            PointerDevice::Pen,
            PointerEventKind::Move,
            Point::new(1.0, 1.0),
        )
        .with_delta(PointerDelta::new(4.0, -3.0))
        .with_tilt(PointerTilt::new(-20.0, 35.0))
        .with_rotation(90.0)
        .with_primary(false)
        .with_hover_capability(true);

        assert_eq!(pen.delta().magnitude(), 5.0);
        assert_eq!(pen.tilt().x_degrees, -20.0);
        assert_eq!(pen.tilt().y_degrees, 35.0);
        assert_eq!(pen.rotation(), 90.0);
        assert!(!pen.is_primary(), "the second contact of a gesture");
        assert!(
            pen.hover_capable(),
            "§2.4: pen hover depends on device capability, so the platform opts in"
        );
    }

    /// A barrel twist is an angle on a circle: 370° and 10° are the same
    /// rotation, and a negative one is the same as its positive complement.
    #[test]
    fn rotation_is_normalized_into_one_turn() {
        assert_eq!(base().with_rotation(370.0).rotation(), 10.0);
        assert_eq!(base().with_rotation(-90.0).rotation(), 270.0);
        assert_eq!(base().with_rotation(360.0).rotation(), 0.0);
    }

    /// §2.4: "Touch does not synthesize persistent hover." — a contact reports
    /// no hover capability without anybody having to remember to say so.
    #[test]
    fn hover_capability_defaults_from_the_device() {
        let contact = PointerEvent::new(
            PointerId::touch(0),
            PointerDevice::Touch,
            PointerEventKind::Down,
            Point::new(0.0, 0.0),
        );
        assert!(!contact.hover_capable());
        assert!(base().hover_capable(), "a mouse drives a resting cursor");
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
