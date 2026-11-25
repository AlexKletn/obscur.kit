//! The [`Shortcut`] key+modifier combination and its [`Modifiers`] bitset and
//! [`Modifier`] members (spec §6.7).

use crate::command::LogicalKey;

use super::{Modifier, Modifiers};

#[doc = include_str!("../../docs/api/command/shortcut.doc/Shortcut.md")]
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

    #[doc = include_str!("../../docs/api/command/shortcut.doc/matches.md")]
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
