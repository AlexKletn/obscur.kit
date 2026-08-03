#![doc = include_str!("shortcut.doc/_module.md")]

use crate::command::LogicalKey;

#[doc = include_str!("shortcut.doc/Modifier.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    #[doc = include_str!("shortcut.doc/Ctrl.md")]
    Ctrl,
    #[doc = include_str!("shortcut.doc/Alt.md")]
    Alt,
    #[doc = include_str!("shortcut.doc/Shift.md")]
    Shift,
    #[doc = include_str!("shortcut.doc/Meta.md")]
    Meta,
}

impl Modifier {
    #[doc = include_str!("shortcut.doc/mask.md")]
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

#[doc = include_str!("shortcut.doc/Modifiers.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers(u8);

impl Modifiers {
    #[doc = include_str!("shortcut.doc/CTRL.md")]
    pub const CTRL: u8 = 1;
    #[doc = include_str!("shortcut.doc/ALT.md")]
    pub const ALT: u8 = 2;
    #[doc = include_str!("shortcut.doc/SHIFT.md")]
    pub const SHIFT: u8 = 4;
    #[doc = include_str!("shortcut.doc/META.md")]
    pub const META: u8 = 8;

    #[doc = include_str!("shortcut.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self(0)
    }

    #[doc = include_str!("shortcut.doc/with.md")]
    #[must_use]
    pub const fn with(self, modifier: Modifier) -> Self {
        Self(self.0 | modifier.mask())
    }

    #[doc = include_str!("shortcut.doc/without.md")]
    #[must_use]
    pub const fn without(self, modifier: Modifier) -> Self {
        Self(self.0 & !modifier.mask())
    }

    #[doc = include_str!("shortcut.doc/contains.md")]
    #[must_use]
    pub const fn contains(&self, modifier: Modifier) -> bool {
        self.0 & modifier.mask() != 0
    }

    #[doc = include_str!("shortcut.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[doc = include_str!("shortcut.doc/bits.md")]
    #[must_use]
    pub const fn bits(&self) -> u8 {
        self.0
    }
}

#[doc = include_str!("shortcut.doc/Shortcut.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    #[doc = include_str!("shortcut.doc/key.md")]
    key: LogicalKey,
    #[doc = include_str!("shortcut.doc/modifiers.md")]
    modifiers: Modifiers,
}

impl Shortcut {
    #[doc = include_str!("shortcut.doc/new_2.md")]
    #[must_use]
    pub const fn new(key: LogicalKey) -> Self {
        Self {
            key,
            modifiers: Modifiers::new(),
        }
    }

    #[doc = include_str!("shortcut.doc/with_modifiers.md")]
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    #[doc = include_str!("shortcut.doc/with_2.md")]
    #[must_use]
    pub const fn with(mut self, modifier: Modifier) -> Self {
        self.modifiers = self.modifiers.with(modifier);
        self
    }

    #[doc = include_str!("shortcut.doc/key_2.md")]
    #[must_use]
    pub const fn key(&self) -> LogicalKey {
        self.key
    }

    #[doc = include_str!("shortcut.doc/modifiers_2.md")]
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    #[doc = include_str!("shortcut.doc/matches.md")]
    #[must_use]
    pub fn matches(&self, key: LogicalKey, modifiers: Modifiers) -> bool {
        self.key == key && self.modifiers == modifiers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::NamedKey;

    #[test]
    fn modifier_masks_are_distinct_bits() {
        assert_eq!(Modifier::Ctrl.mask(), Modifiers::CTRL);
        assert_eq!(Modifier::Alt.mask(), Modifiers::ALT);
        assert_eq!(Modifier::Shift.mask(), Modifiers::SHIFT);
        assert_eq!(Modifier::Meta.mask(), Modifiers::META);
    }

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

    #[test]
    fn shortcut_accessors() {
        let sc = Shortcut::new(LogicalKey::character('s')).with(Modifier::Ctrl);
        assert_eq!(sc.key(), LogicalKey::character('s'));
        assert!(sc.modifiers().contains(Modifier::Ctrl));
    }

    #[test]
    fn matches_requires_exact_key_and_modifiers() {
        let ctrl = Modifiers::new().with(Modifier::Ctrl);
        let sc = Shortcut::new(LogicalKey::character('s')).with_modifiers(ctrl);

        assert!(sc.matches(LogicalKey::character('s'), ctrl));
        assert!(!sc.matches(LogicalKey::character('x'), ctrl));
        assert!(!sc.matches(LogicalKey::character('s'), Modifiers::new()));
        let ctrl_shift = ctrl.with(Modifier::Shift);
        assert!(!sc.matches(LogicalKey::character('s'), ctrl_shift));
    }

    #[test]
    fn matches_named_keys() {
        let sc = Shortcut::new(LogicalKey::named(NamedKey::Enter));
        assert!(sc.matches(LogicalKey::named(NamedKey::Enter), Modifiers::new()));
        assert!(!sc.matches(LogicalKey::named(NamedKey::Escape), Modifiers::new()));
    }
}
