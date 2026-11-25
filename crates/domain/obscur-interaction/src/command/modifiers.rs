use super::Modifier;

#[doc = include_str!("../../docs/api/command/shortcut.doc/Modifiers.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers(u8);

impl Modifiers {
    /// Bit for [`Modifier::Ctrl`].
    pub const CTRL: u8 = 1;
    /// Bit for [`Modifier::Alt`].
    pub const ALT: u8 = 2;
    /// Bit for [`Modifier::Shift`].
    pub const SHIFT: u8 = 4;
    /// Bit for [`Modifier::Meta`].
    pub const META: u8 = 8;

    /// Creates an empty modifier set (no modifiers held).
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Returns this set with `modifier` added.
    #[must_use]
    pub const fn with(self, modifier: Modifier) -> Self {
        Self(self.0 | modifier.mask())
    }

    /// Returns this set with `modifier` removed (a no-op when it was absent).
    #[must_use]
    pub const fn without(self, modifier: Modifier) -> Self {
        Self(self.0 & !modifier.mask())
    }

    /// Returns whether `modifier` is present in this set.
    #[must_use]
    pub const fn contains(&self, modifier: Modifier) -> bool {
        self.0 & modifier.mask() != 0
    }

    /// Returns whether no modifiers are held.
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
    fn modifiers_with_contains_and_is_empty() {
        let empty = Modifiers::new();
        assert!(empty.is_empty());
        assert!(Modifiers::default().is_empty());
        assert!(!empty.contains(Modifier::Ctrl));

        let m = Modifiers::new().with(Modifier::Ctrl).with(Modifier::Shift);
        assert!(!m.is_empty());
        assert!(m.contains(Modifier::Ctrl));
        assert!(m.contains(Modifier::Shift));
        assert!(!m.contains(Modifier::Alt));
        assert_eq!(m.bits(), Modifiers::CTRL | Modifiers::SHIFT);
    }

    #[test]
    fn without_removes_a_held_modifier_and_is_idempotent() {
        let m = Modifiers::new().with(Modifier::Ctrl).with(Modifier::Shift);
        let dropped = m.without(Modifier::Ctrl);
        assert!(!dropped.contains(Modifier::Ctrl));
        assert!(dropped.contains(Modifier::Shift));
        // Removing an absent modifier is a no-op.
        assert_eq!(dropped.without(Modifier::Ctrl), dropped);
        assert!(Modifiers::new().without(Modifier::Alt).is_empty());
    }
}
