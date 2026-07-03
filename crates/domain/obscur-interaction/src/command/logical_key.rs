//! The [`LogicalKey`] layout-dependent produced key and its [`NamedKey`]
//! members (spec §6.7).

/// A non-character key identified by name (spec §6.7).
///
/// These are the keys that do not produce a character: editing, navigation, and
/// function keys. Character-producing keys are carried by
/// [`LogicalKey::Character`] instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    /// The `Enter` / `Return` key.
    Enter,
    /// The `Escape` key.
    Escape,
    /// The `Tab` key.
    Tab,
    /// The space key (as a named key rather than a `' '` character).
    Space,
    /// The `Backspace` key.
    Backspace,
    /// The forward `Delete` key.
    Delete,
    /// The up arrow key.
    ArrowUp,
    /// The down arrow key.
    ArrowDown,
    /// The left arrow key.
    ArrowLeft,
    /// The right arrow key.
    ArrowRight,
    /// The `Home` key.
    Home,
    /// The `End` key.
    End,
    /// The `Page Up` key.
    PageUp,
    /// The `Page Down` key.
    PageDown,
    /// The `F1` function key.
    F1,
    /// The `F2` function key.
    F2,
    /// The `F3` function key.
    F3,
    /// The `F4` function key.
    F4,
    /// The `F5` function key.
    F5,
    /// The `F6` function key.
    F6,
    /// The `F7` function key.
    F7,
    /// The `F8` function key.
    F8,
    /// The `F9` function key.
    F9,
    /// The `F10` function key.
    F10,
    /// The `F11` function key.
    F11,
    /// The `F12` function key.
    F12,
}

/// A layout-dependent produced key (spec §6.7).
///
/// A logical key is what the active keyboard layout produces when a physical key
/// is pressed: the character `'a'`, `'/'`, or `'?'`, or a
/// [named](NamedKey) non-character key such as `Enter`. Logical keys are the
/// right surface for text entry and shortcuts, which follow the user's layout;
/// for layout-independent spatial bindings use
/// [`PhysicalKey`](crate::command::PhysicalKey).
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
