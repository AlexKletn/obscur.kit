//! The [`Shortcut`] key+modifier combination and its [`Modifiers`] bitset and
//! [`Modifier`] members (spec §6.7).

use crate::command::LogicalKey;

/// A single keyboard modifier (spec §6.7).
///
/// Each variant maps to one bit in a [`Modifiers`] bitset via
/// [`Modifier::mask`].
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

/// The set of modifiers held for a shortcut (spec §6.7).
///
/// Backed by a `u8` bitset; each [`Modifier`] contributes one bit. An empty set
/// means no modifiers are held.
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

/// A key and modifier combination that triggers a command (spec §6.7).
///
/// A shortcut pairs a [`LogicalKey`] with a [`Modifiers`] set. It uses logical
/// (layout-dependent) keys so bindings follow the user's keyboard layout. A
/// shortcut with no modifiers is a bare key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// The logical key that triggers the shortcut.
    key: LogicalKey,
    /// The modifiers that must be held.
    modifiers: Modifiers,
}

impl Shortcut {
    /// Creates a shortcut for `key` with no modifiers.
    #[must_use]
    pub const fn new(key: LogicalKey) -> Self {
        Self {
            key,
            modifiers: Modifiers::new(),
        }
    }

    /// Returns this shortcut with its modifier set replaced by `modifiers`.
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Returns this shortcut with `modifier` added to its modifier set.
    #[must_use]
    pub const fn with(mut self, modifier: Modifier) -> Self {
        self.modifiers = self.modifiers.with(modifier);
        self
    }

    /// Returns the logical key that triggers this shortcut.
    #[must_use]
    pub const fn key(&self) -> LogicalKey {
        self.key
    }

    /// Returns the modifiers that must be held.
    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Returns whether this shortcut is triggered by `key` with exactly
    /// `modifiers` held.
    ///
    /// Matching requires an exact modifier set: an extra held modifier, or a
    /// missing one, does not match.
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
        // Wrong key.
        assert!(!sc.matches(LogicalKey::character('x'), ctrl));
        // Missing modifier.
        assert!(!sc.matches(LogicalKey::character('s'), Modifiers::new()));
        // Extra modifier.
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
