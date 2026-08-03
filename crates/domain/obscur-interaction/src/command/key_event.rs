#![doc = include_str!("key_event.doc/_module.md")]

use crate::command::{LogicalKey, Modifiers, PhysicalKey};
use crate::event::EventTrust;

#[doc = include_str!("key_event.doc/KeyEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    #[doc = include_str!("key_event.doc/physical.md")]
    pub physical: PhysicalKey,
    #[doc = include_str!("key_event.doc/logical.md")]
    pub logical: Option<LogicalKey>,
    #[doc = include_str!("key_event.doc/pressed.md")]
    pub pressed: bool,
    #[doc = include_str!("key_event.doc/modifiers.md")]
    pub modifiers: Modifiers,
    #[doc = include_str!("key_event.doc/repeat.md")]
    pub repeat: bool,
    #[doc = include_str!("key_event.doc/timestamp_micros.md")]
    pub timestamp_micros: u64,
    #[doc = include_str!("key_event.doc/trust.md")]
    pub trust: EventTrust,
}

impl KeyEvent {
    #[doc = include_str!("key_event.doc/new.md")]
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

    #[doc = include_str!("key_event.doc/press.md")]
    #[must_use]
    pub const fn press(physical: PhysicalKey, logical: Option<LogicalKey>) -> Self {
        Self::new(physical, logical, true)
    }

    #[doc = include_str!("key_event.doc/release.md")]
    #[must_use]
    pub const fn release(physical: PhysicalKey, logical: Option<LogicalKey>) -> Self {
        Self::new(physical, logical, false)
    }

    #[doc = include_str!("key_event.doc/physical_2.md")]
    #[must_use]
    pub const fn physical(&self) -> PhysicalKey {
        self.physical
    }

    #[doc = include_str!("key_event.doc/logical_2.md")]
    #[must_use]
    pub const fn logical(&self) -> Option<LogicalKey> {
        self.logical
    }

    #[doc = include_str!("key_event.doc/pressed_2.md")]
    #[must_use]
    pub const fn pressed(&self) -> bool {
        self.pressed
    }

    #[doc = include_str!("key_event.doc/with_modifiers.md")]
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    #[doc = include_str!("key_event.doc/modifiers_2.md")]
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    #[doc = include_str!("key_event.doc/as_repeat.md")]
    #[must_use]
    pub const fn as_repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    #[doc = include_str!("key_event.doc/is_repeat.md")]
    #[must_use]
    pub const fn is_repeat(&self) -> bool {
        self.repeat
    }

    #[doc = include_str!("key_event.doc/with_timestamp.md")]
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    #[doc = include_str!("key_event.doc/timestamp.md")]
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    #[doc = include_str!("key_event.doc/as_trusted.md")]
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    #[doc = include_str!("key_event.doc/trust_2.md")]
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    #[doc = include_str!("key_event.doc/is_trusted.md")]
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
