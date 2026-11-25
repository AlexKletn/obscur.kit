//! The [`KeyEvent`] payload carried by a keyboard key transition (spec §6.7).

use crate::command::{LogicalKey, Modifiers, PhysicalKey};
use crate::event::EventTrust;

#[doc = include_str!("../../docs/api/command/key_event.doc/KeyEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    /// The layout-independent physical key position.
    pub physical: PhysicalKey,
    /// The layout-dependent produced key, if the layout resolves one.
    pub logical: Option<LogicalKey>,
    /// Whether this is a press (`true`) or a release (`false`).
    pub pressed: bool,
    #[doc = include_str!("../../docs/api/command/key_event.doc/modifiers.md")]
    pub modifiers: Modifiers,
    #[doc = include_str!("../../docs/api/command/key_event.doc/repeat.md")]
    pub repeat: bool,
    #[doc = include_str!("../../docs/api/command/key_event.doc/timestamp_micros.md")]
    pub timestamp_micros: u64,
    #[doc = include_str!("../../docs/api/command/key_event.doc/trust.md")]
    pub trust: EventTrust,
}

impl KeyEvent {
    /// Creates a key event for `physical` with an optional produced `logical`
    /// key and a `pressed` flag.
    #[must_use]
    pub const fn new(physical: PhysicalKey, logical: Option<LogicalKey>, pressed: bool) -> Self {
        Self {
            physical,
            logical,
            pressed,
            modifiers: Modifiers::new(),
            repeat: false,
            timestamp_micros: 0,
            trust: EventTrust::Untrusted,
        }
    }

    /// Creates a key **press** event.
    #[must_use]
    pub const fn press(physical: PhysicalKey, logical: Option<LogicalKey>) -> Self {
        Self::new(physical, logical, true)
    }

    /// Creates a key **release** event.
    #[must_use]
    pub const fn release(physical: PhysicalKey, logical: Option<LogicalKey>) -> Self {
        Self::new(physical, logical, false)
    }

    /// Returns the layout-independent physical key position.
    #[must_use]
    pub const fn physical(&self) -> PhysicalKey {
        self.physical
    }

    /// Returns the layout-dependent produced key, if any.
    #[must_use]
    pub const fn logical(&self) -> Option<LogicalKey> {
        self.logical
    }

    /// Returns whether this is a press (`true`) or a release (`false`).
    #[must_use]
    pub const fn pressed(&self) -> bool {
        self.pressed
    }

    /// Returns the event with `modifiers` replaced.
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// The chord held when the key transitioned.
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Returns the event marked as an auto-repeat.
    #[must_use]
    pub const fn as_repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    /// Whether the platform generated this press by auto-repeat.
    #[must_use]
    pub const fn is_repeat(&self) -> bool {
        self.repeat
    }

    /// Returns the event stamped with a host timestamp, in microseconds.
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    /// The host timestamp in microseconds, or zero if unstamped.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    #[doc = include_str!("../../docs/api/command/key_event.doc/as_trusted.md")]
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    /// Whether this key came from a real device or was synthesised.
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    /// Whether this key came from a real device.
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Modifier, NamedKey};

    #[test]
    fn new_records_all_three_parts() {
        let event = KeyEvent::new(PhysicalKey::KeyA, Some(LogicalKey::character('a')), true);
        assert_eq!(event.physical(), PhysicalKey::KeyA);
        assert_eq!(event.logical(), Some(LogicalKey::character('a')));
        assert!(event.pressed());
    }

    #[test]
    fn press_and_release_constructors() {
        let down = KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter)));
        assert!(down.pressed());
        let up = KeyEvent::release(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter)));
        assert!(!up.pressed());
    }

    #[test]
    fn logical_is_optional() {
        // A bare modifier has no modeled logical key.
        let shift = KeyEvent::press(PhysicalKey::ShiftLeft, None);
        assert_eq!(shift.logical(), None);
        assert_eq!(shift.physical(), PhysicalKey::ShiftLeft);
    }

    #[test]
    fn a_fresh_event_is_untrusted_unstamped_and_not_a_repeat() {
        // Fails closed, exactly as `PointerEvent::new` does: trust is conferred
        // at the platform boundary and never assumed, and a host that stamps no
        // clock gets zero rather than a guess.
        let event = KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a')));
        assert!(!event.is_trusted());
        assert_eq!(event.trust(), EventTrust::Untrusted);
        assert_eq!(event.timestamp(), 0);
        assert!(!event.is_repeat());
        assert!(event.modifiers().is_empty());
    }

    #[test]
    fn the_envelope_carries_the_chord_the_key_was_pressed_under() {
        // The reason this field exists: a handler holding the event must be able
        // to answer "was Ctrl down?" without asking the runtime for ambient
        // state it may since have changed.
        let chord = Modifiers::new().with(Modifier::Ctrl).with(Modifier::Shift);
        let event = KeyEvent::press(PhysicalKey::KeyS, Some(LogicalKey::character('s')))
            .with_modifiers(chord);
        assert_eq!(event.modifiers(), chord);
        assert!(event.modifiers().contains(Modifier::Ctrl));
    }

    #[test]
    fn every_builder_leaves_the_other_fields_alone() {
        let base = KeyEvent::press(PhysicalKey::KeyA, Some(LogicalKey::character('a')));
        let built = base
            .with_modifiers(Modifiers::new().with(Modifier::Alt))
            .as_repeat()
            .with_timestamp(12_345)
            .as_trusted();
        assert_eq!(built.physical(), base.physical());
        assert_eq!(built.logical(), base.logical());
        assert_eq!(built.pressed(), base.pressed());
        assert!(built.is_repeat());
        assert_eq!(built.timestamp(), 12_345);
        assert!(built.is_trusted());
    }
}
