use super::PointerButtons;

#[doc = include_str!("../../docs/api/pointer/pointer_buttons.doc/PointerButton.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The primary button (typically left mouse / main contact).
    Primary,
    /// The secondary button (typically right mouse / context).
    Secondary,
    /// The middle button (typically the mouse wheel press).
    Middle,
    /// The back navigation button.
    Back,
    /// The forward navigation button.
    Forward,
}

impl PointerButton {
    /// Returns the single-bit mask this button occupies within a
    /// [`PointerButtons`] bitset.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_mask_matches_constants() {
        assert_eq!(PointerButton::Primary.mask(), PointerButtons::PRIMARY);
        assert_eq!(PointerButton::Secondary.mask(), PointerButtons::SECONDARY);
        assert_eq!(PointerButton::Middle.mask(), PointerButtons::MIDDLE);
        assert_eq!(PointerButton::Back.mask(), PointerButtons::BACK);
        assert_eq!(PointerButton::Forward.mask(), PointerButtons::FORWARD);
    }
}
