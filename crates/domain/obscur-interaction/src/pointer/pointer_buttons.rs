#![doc = include_str!("pointer_buttons.doc/_module.md")]

#[doc = include_str!("pointer_buttons.doc/PointerButton.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    #[doc = include_str!("pointer_buttons.doc/Primary.md")]
    Primary,
    #[doc = include_str!("pointer_buttons.doc/Secondary.md")]
    Secondary,
    #[doc = include_str!("pointer_buttons.doc/Middle.md")]
    Middle,
    #[doc = include_str!("pointer_buttons.doc/Back.md")]
    Back,
    #[doc = include_str!("pointer_buttons.doc/Forward.md")]
    Forward,
}

impl PointerButton {
    #[doc = include_str!("pointer_buttons.doc/mask.md")]
    #[must_use]
    pub const fn mask(self) -> u8 {
        match self {
            Self::Primary => PointerButtons::PRIMARY,
            Self::Secondary => PointerButtons::SECONDARY,
            Self::Middle => PointerButtons::MIDDLE,
            Self::Back => PointerButtons::BACK,
            Self::Forward => PointerButtons::FORWARD,
        }
    }
}

#[doc = include_str!("pointer_buttons.doc/PointerButtons.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PointerButtons(u8);

impl PointerButtons {
    #[doc = include_str!("pointer_buttons.doc/PRIMARY.md")]
    pub const PRIMARY: u8 = 1;
    #[doc = include_str!("pointer_buttons.doc/SECONDARY.md")]
    pub const SECONDARY: u8 = 2;
    #[doc = include_str!("pointer_buttons.doc/MIDDLE.md")]
    pub const MIDDLE: u8 = 4;
    #[doc = include_str!("pointer_buttons.doc/BACK.md")]
    pub const BACK: u8 = 8;
    #[doc = include_str!("pointer_buttons.doc/FORWARD.md")]
    pub const FORWARD: u8 = 16;

    #[doc = include_str!("pointer_buttons.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    #[doc = include_str!("pointer_buttons.doc/from_bits.md")]
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    #[doc = include_str!("pointer_buttons.doc/press.md")]
    pub const fn press(&mut self, button: PointerButton) {
        self.0 |= button.mask();
    }

    #[doc = include_str!("pointer_buttons.doc/release.md")]
    pub const fn release(&mut self, button: PointerButton) {
        self.0 &= !button.mask();
    }

    #[doc = include_str!("pointer_buttons.doc/contains.md")]
    #[must_use]
    pub const fn contains(&self, button: PointerButton) -> bool {
        self.0 & button.mask() != 0
    }

    #[doc = include_str!("pointer_buttons.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[doc = include_str!("pointer_buttons.doc/bits.md")]
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

    #[test]
    fn button_mask_matches_constants() {
        assert_eq!(PointerButton::Primary.mask(), PointerButtons::PRIMARY);
        assert_eq!(PointerButton::Secondary.mask(), PointerButtons::SECONDARY);
        assert_eq!(PointerButton::Middle.mask(), PointerButtons::MIDDLE);
        assert_eq!(PointerButton::Back.mask(), PointerButtons::BACK);
        assert_eq!(PointerButton::Forward.mask(), PointerButtons::FORWARD);
    }
}
