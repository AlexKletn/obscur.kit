//! The [`LogicalKey`] layout-dependent produced key and its [`NamedKey`]
//! members (spec §6.7).

use super::NamedKey;

#[doc = include_str!("../../docs/api/command/logical_key.doc/LogicalKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalKey {
    /// A character produced by the active layout (e.g. `'a'`, `'/'`, `'?'`).
    Character(char),
    /// A non-character key identified by name.
    Named(NamedKey),
}

impl LogicalKey {
    /// Creates a logical key for a produced `character`.
    #[must_use]
    pub const fn character(character: char) -> Self {
        Self::Character(character)
    }

    /// Creates a logical key for a `named` non-character key.
    #[must_use]
    pub const fn named(named: NamedKey) -> Self {
        Self::Named(named)
    }

    /// Returns the produced character for a [`LogicalKey::Character`], else
    /// `None`.
    #[must_use]
    pub const fn as_character(&self) -> Option<char> {
        match self {
            Self::Character(c) => Some(*c),
            Self::Named(_) => None,
        }
    }

    /// Returns whether this is a [`LogicalKey::Named`] key.
    #[must_use]
    pub const fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }

    /// Returns the [`NamedKey`] for a [`LogicalKey::Named`], else `None`.
    #[must_use]
    pub const fn named_key(&self) -> Option<NamedKey> {
        match self {
            Self::Named(k) => Some(*k),
            Self::Character(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_constructor_and_accessor() {
        let key = LogicalKey::character('a');
        assert_eq!(key, LogicalKey::Character('a'));
        assert_eq!(key.as_character(), Some('a'));
        assert!(!key.is_named());
        assert_eq!(key.named_key(), None);
    }

    #[test]
    fn named_constructor_and_accessor() {
        let key = LogicalKey::named(NamedKey::Enter);
        assert_eq!(key, LogicalKey::Named(NamedKey::Enter));
        assert_eq!(key.as_character(), None);
        assert!(key.is_named());
        assert_eq!(key.named_key(), Some(NamedKey::Enter));
    }

    #[test]
    fn punctuation_characters_are_supported() {
        assert_eq!(LogicalKey::character('/').as_character(), Some('/'));
        assert_eq!(LogicalKey::character('?').as_character(), Some('?'));
    }
}
