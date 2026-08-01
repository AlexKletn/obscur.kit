#![doc = include_str!("key_event.doc/_module.md")]

use crate::command::{LogicalKey, PhysicalKey};

#[doc = include_str!("key_event.doc/KeyEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    #[doc = include_str!("key_event.doc/physical.md")]
    pub physical: PhysicalKey,
    #[doc = include_str!("key_event.doc/logical.md")]
    pub logical: Option<LogicalKey>,
    #[doc = include_str!("key_event.doc/pressed.md")]
    pub pressed: bool,
}

impl KeyEvent {
    #[doc = include_str!("key_event.doc/new.md")]
    #[must_use]
    pub const fn new(physical: PhysicalKey, logical: Option<LogicalKey>, pressed: bool) -> Self {
        Self {
            physical,
            logical,
            pressed,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::NamedKey;

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
}
