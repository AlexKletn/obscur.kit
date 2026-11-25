//! The [`PointerButtons`] pressed-button bitset and its [`PointerButton`]
//! members (spec §4.7).

use super::PointerButton;

#[doc = include_str!("../../docs/api/pointer/pointer_buttons.doc/PointerButtons.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PointerButtons(u8);

impl PointerButtons {
    /// Bit for [`PointerButton::Primary`].
    pub const PRIMARY: u8 = 1;
    /// Bit for [`PointerButton::Secondary`].
    pub const SECONDARY: u8 = 2;
    /// Bit for [`PointerButton::Middle`].
    pub const MIDDLE: u8 = 4;
    /// Bit for [`PointerButton::Back`].
    pub const BACK: u8 = 8;
    /// Bit for [`PointerButton::Forward`].
    pub const FORWARD: u8 = 16;

    /// Creates an empty button set (no buttons pressed).
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Creates a button set from a raw `u8` bitmask.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    /// Marks `button` as pressed.
    pub const fn press(&mut self, button: PointerButton) {
        self.0 |= button.mask();
    }

    /// Marks `button` as released.
    pub const fn release(&mut self, button: PointerButton) {
        self.0 &= !button.mask();
    }

    /// Returns whether `button` is currently pressed.
    #[must_use]
    pub const fn contains(&self, button: PointerButton) -> bool {
        self.0 & button.mask() != 0
    }

    /// Returns whether no buttons are pressed.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns the raw `u8` bitmask backing this set.
    #[must_use]
    pub const fn bits(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_default_are_empty() {
        assert!(PointerButtons::new().is_empty());
        assert!(PointerButtons::default().is_empty());
        assert_eq!(PointerButtons::new().bits(), 0);
    }

    #[test]
    fn press_release_and_contains() {
        let mut b = PointerButtons::new();
        assert!(!b.contains(PointerButton::Primary));

        b.press(PointerButton::Primary);
        assert!(b.contains(PointerButton::Primary));
        assert!(!b.is_empty());

        b.press(PointerButton::Secondary);
        assert!(b.contains(PointerButton::Secondary));
        assert!(b.contains(PointerButton::Primary));

        b.release(PointerButton::Primary);
        assert!(!b.contains(PointerButton::Primary));
        assert!(b.contains(PointerButton::Secondary));
    }

    #[test]
    fn from_bits_round_trips() {
        let b = PointerButtons::from_bits(PointerButtons::MIDDLE);
        assert!(b.contains(PointerButton::Middle));
        assert_eq!(b.bits(), PointerButtons::MIDDLE);
    }
}
