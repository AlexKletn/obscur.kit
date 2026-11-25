use super::Modifiers;

#[doc = include_str!("../../docs/api/command/shortcut.doc/Modifier.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    /// The `Control` modifier.
    Ctrl,
    /// The `Alt` / `Option` modifier.
    Alt,
    /// The `Shift` modifier.
    Shift,
    /// The `Meta` / `Super` / `Command` modifier.
    Meta,
}

impl Modifier {
    /// Returns the single-bit mask this modifier occupies within a
    /// [`Modifiers`] bitset.
    #[must_use]
    pub const fn mask(self) -> u8 {
        match self {
            Self::Ctrl => Modifiers::CTRL,
            Self::Alt => Modifiers::ALT,
            Self::Shift => Modifiers::SHIFT,
            Self::Meta => Modifiers::META,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_masks_are_distinct_bits() {
        assert_eq!(Modifier::Ctrl.mask(), Modifiers::CTRL);
        assert_eq!(Modifier::Alt.mask(), Modifiers::ALT);
        assert_eq!(Modifier::Shift.mask(), Modifiers::SHIFT);
        assert_eq!(Modifier::Meta.mask(), Modifiers::META);
    }
}
