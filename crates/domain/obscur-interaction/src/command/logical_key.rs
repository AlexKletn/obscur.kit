#![doc = include_str!("logical_key.doc/_module.md")]

#[doc = include_str!("logical_key.doc/NamedKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    #[doc = include_str!("logical_key.doc/Enter.md")]
    Enter,
    #[doc = include_str!("logical_key.doc/Escape.md")]
    Escape,
    #[doc = include_str!("logical_key.doc/Tab.md")]
    Tab,
    #[doc = include_str!("logical_key.doc/Space.md")]
    Space,
    #[doc = include_str!("logical_key.doc/Backspace.md")]
    Backspace,
    #[doc = include_str!("logical_key.doc/Delete.md")]
    Delete,
    #[doc = include_str!("logical_key.doc/ArrowUp.md")]
    ArrowUp,
    #[doc = include_str!("logical_key.doc/ArrowDown.md")]
    ArrowDown,
    #[doc = include_str!("logical_key.doc/ArrowLeft.md")]
    ArrowLeft,
    #[doc = include_str!("logical_key.doc/ArrowRight.md")]
    ArrowRight,
    #[doc = include_str!("logical_key.doc/Home.md")]
    Home,
    #[doc = include_str!("logical_key.doc/End.md")]
    End,
    #[doc = include_str!("logical_key.doc/PageUp.md")]
    PageUp,
    #[doc = include_str!("logical_key.doc/PageDown.md")]
    PageDown,
    #[doc = include_str!("logical_key.doc/F1.md")]
    F1,
    #[doc = include_str!("logical_key.doc/F2.md")]
    F2,
    #[doc = include_str!("logical_key.doc/F3.md")]
    F3,
    #[doc = include_str!("logical_key.doc/F4.md")]
    F4,
    #[doc = include_str!("logical_key.doc/F5.md")]
    F5,
    #[doc = include_str!("logical_key.doc/F6.md")]
    F6,
    #[doc = include_str!("logical_key.doc/F7.md")]
    F7,
    #[doc = include_str!("logical_key.doc/F8.md")]
    F8,
    #[doc = include_str!("logical_key.doc/F9.md")]
    F9,
    #[doc = include_str!("logical_key.doc/F10.md")]
    F10,
    #[doc = include_str!("logical_key.doc/F11.md")]
    F11,
    #[doc = include_str!("logical_key.doc/F12.md")]
    F12,
}

#[doc = include_str!("logical_key.doc/LogicalKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalKey {
    #[doc = include_str!("logical_key.doc/Character.md")]
    Character(char),
    #[doc = include_str!("logical_key.doc/Named.md")]
    Named(NamedKey),
}

impl LogicalKey {
    #[doc = include_str!("logical_key.doc/character.md")]
    #[must_use]
    pub const fn character(character: char) -> Self {
        Self::Character(character)
    }

    #[doc = include_str!("logical_key.doc/named.md")]
    #[must_use]
    pub const fn named(named: NamedKey) -> Self {
        Self::Named(named)
    }

    #[doc = include_str!("logical_key.doc/as_character.md")]
    #[must_use]
    pub const fn as_character(&self) -> Option<char> {
        match self {
            Self::Character(c) => Some(*c),
            Self::Named(_) => None,
        }
    }

    #[doc = include_str!("logical_key.doc/is_named.md")]
    #[must_use]
    pub const fn is_named(&self) -> bool {
        matches!(self, Self::Named(_))
    }

    #[doc = include_str!("logical_key.doc/named_key.md")]
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
