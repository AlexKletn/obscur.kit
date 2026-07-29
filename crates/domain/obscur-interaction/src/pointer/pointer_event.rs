//! The [`PointerEvent`] payload carried by a hit-tested pointer event and its
//! [`PointerEventKind`] discriminant (spec §4.7).

use obscur_geometry::Point;

use super::{PointerButtons, PointerDelta, PointerDevice, PointerId, PointerTilt};
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

/// A single pointer event delivered to a hit-tested target (spec §2.4, §4.7).
///
/// Each event describes one pointer's state at one instant. §2.4 enumerates what
/// that state is, and this struct is that list:
///
/// > The pointer model supports mouse, touch, pen and trackpad-like sources:
/// >
/// > - pointer ID;
/// > - device type;
/// > - buttons;
/// > - logical position and delta;
/// > - pressure;
/// > - tilt;
/// > - rotation;
/// > - primary-pointer status;
/// > - hover capability.
///
/// So: which [`pointer`](Self::pointer) and [`device`](Self::device) produced it,
/// the [`kind`](Self::kind) of change, the [`position`](Self::position) in the
/// target's coordinate space and the [`delta`](Self::delta) that brought it there,
/// the [`buttons`](Self::buttons) held *after* this event, the contact
/// [`pressure`](Self::pressure) (`0.0` for hover or a device without pressure),
/// the stylus [`tilt`](Self::tilt) and [`rotation`](Self::rotation), whether this
/// is the [`primary`](Self::is_primary) pointer, whether the device
/// [`hovers`](Self::hover_capable), and a logical [`timestamp`](Self::timestamp)
/// supplied by the caller — this crate never reads a clock.
///
/// **Four of those were absent** until the §2.3/§2.4 delivery pass: delta, tilt,
/// rotation and primary-pointer status were listed by the spec and modeled
/// nowhere, so no handler could have read them however the event reached it. They
/// default to "nothing to report" ([`PointerDelta::ZERO`],
/// [`PointerTilt::UPRIGHT`], no twist, primary) so a producer that knows nothing
/// about them keeps working unchanged and one that does opts in field by field.
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
    /// §2.4's delta: how far this pointer moved since its own previous event.
    pub delta: PointerDelta,
    /// The buttons held down after this event.
    pub buttons: PointerButtons,
    /// The contact pressure in `0.0..=1.0` (`0.0` for hover / no contact).
    pub pressure: f32,
    /// §2.4's tilt: the angle the device is held at, per axis.
    pub tilt: PointerTilt,
    /// §2.4's rotation: the stylus barrel's twist about its own axis, in
    /// `0..360` degrees.
    pub rotation: f32,
    /// §2.4's primary-pointer status: whether this pointer is the primary one of
    /// its kind (the mouse always is; of several simultaneous contacts, only the
    /// first).
    pub primary: bool,
    /// §2.4's hover capability: whether this device can be *over* a target
    /// without touching it. False for a touchscreen contact, which is why touch
    /// leaves no persistent hover.
    pub hover_capable: bool,
    /// A logical timestamp in microseconds, supplied by the caller.
    pub timestamp_micros: u64,
    /// Whether this came from a real user gesture (§2.3 trusted/untrusted
    /// origin). Untrusted unless a platform adapter vouched for it.
    pub trust: EventTrust,
}

impl PointerEvent {
    /// Creates a pointer event with no buttons held, zero pressure, no movement,
    /// no tilt or twist, a zero timestamp, and an
    /// [`Untrusted`](EventTrust::Untrusted) origin.
    ///
    /// Untrusted is deliberate, not a placeholder: a synthetic event that forgets
    /// to say what it is must not pass for a real gesture. The one producer of
    /// real input — the platform adapter holding the OS event — opts in with
    /// [`as_trusted`](Self::as_trusted).
    ///
    /// Two of §2.4's fields are defaulted from the *device* rather than from
    /// nothing, because the device already answers them for the common case:
    /// [`hover_capable`](Self::hover_capable) follows
    /// [`PointerDevice::hovers_by_default`] (§2.4: *"Touch does not synthesize
    /// persistent hover. Pen hover depends on device capability."* — so a pen
    /// starts non-hovering and a platform that knows better opts in with
    /// [`with_hover_capability`](Self::with_hover_capability)), and
    /// [`primary`](Self::primary) starts `true`, which is correct for every
    /// single-pointer host and for the *first* contact of a multi-touch one. A
    /// producer that tracks several simultaneous contacts marks the others with
    /// [`with_primary`](Self::with_primary)`(false)`.
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
            pressure: 0.0,
            tilt: PointerTilt::UPRIGHT,
            rotation: 0.0,
            primary: true,
            hover_capable: device.hovers_by_default(),
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

    /// Returns this event with its §2.4 movement delta set (fluent).
    #[must_use]
    pub const fn with_delta(mut self, delta: PointerDelta) -> Self {
        self.delta = delta;
        self
    }

    /// Returns this event with its §2.4 stylus tilt set (fluent).
    #[must_use]
    pub const fn with_tilt(mut self, tilt: PointerTilt) -> Self {
        self.tilt = tilt;
        self
    }

    /// Returns this event with its §2.4 barrel rotation set, normalized into
    /// `0..360` degrees (fluent).
    #[must_use]
    pub fn with_rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees.rem_euclid(360.0);
        self
    }

    /// Returns this event with its §2.4 primary-pointer status set (fluent).
    ///
    /// A multi-touch producer marks every contact but the first as non-primary; a
    /// single-pointer host never calls this, because [`new`](Self::new) already
    /// says `true`.
    #[must_use]
    pub const fn with_primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    /// Returns this event with its §2.4 hover capability set (fluent).
    ///
    /// This is the opt-in §2.4's *"Pen hover depends on device capability"* asks
    /// for: only the platform knows whether the tablet in front of the user
    /// reports a hovering stylus, so only the platform may say so.
    #[must_use]
    pub const fn with_hover_capability(mut self, hover_capable: bool) -> Self {
        self.hover_capable = hover_capable;
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

    /// Returns §2.4's delta: how far this pointer moved since its own previous
    /// event.
    #[must_use]
    pub const fn delta(&self) -> PointerDelta {
        self.delta
    }

    /// Returns §2.4's tilt: the angle the device is held at, per axis.
    #[must_use]
    pub const fn tilt(&self) -> PointerTilt {
        self.tilt
    }

    /// Returns §2.4's rotation: the barrel's twist in `0..360` degrees.
    #[must_use]
    pub const fn rotation(&self) -> f32 {
        self.rotation
    }

    /// Returns §2.4's primary-pointer status.
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.primary
    }

    /// Returns §2.4's hover capability: whether this device can be over a target
    /// without touching it.
    #[must_use]
    pub const fn hover_capable(&self) -> bool {
        self.hover_capable
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
