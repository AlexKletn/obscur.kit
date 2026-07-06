//! The [`TextEdit`] per-keystroke text-input edit descriptor (spec §6.7).

/// A single text-input edit produced by a keystroke on a focused editable
/// (spec §6.7).
///
/// A `TextEdit` is the domain payload the runtime hands to the application when
/// a printable character (or `Backspace`) is typed into a focused text field: it
/// names *what* edit to apply, not the resulting text. The application owns the
/// field's text state and applies the edit itself — mirroring the reactive model
/// where the runtime never owns application state (an `Rc<RefCell<String>>` the
/// view builder reads back), so there is no seed/divergence problem between a
/// runtime-owned buffer and the app's copy.
///
/// It is deliberately minimal — an insertion or a backward delete — the two
/// edits a bare keymap produces. Richer edits (forward delete, word/line
/// motion, selection replacement, paste) are a follow-up and would extend this
/// enum without changing the dispatch path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextEdit {
    /// Insert `char` at the caret — a printable character was typed.
    Insert(char),
    /// Delete the character before the caret — `Backspace`.
    DeleteBackward,
}

impl TextEdit {
    /// Creates an [`Insert`](Self::Insert) edit for `character`.
    #[must_use]
    pub const fn insert(character: char) -> Self {
        Self::Insert(character)
    }

    /// Returns the inserted character for an [`Insert`](Self::Insert), else
    /// `None`.
    #[must_use]
    pub const fn inserted(&self) -> Option<char> {
        match self {
            Self::Insert(c) => Some(*c),
            Self::DeleteBackward => None,
        }
    }

    /// Returns whether this edit is a [`DeleteBackward`](Self::DeleteBackward).
    #[must_use]
    pub const fn is_delete_backward(&self) -> bool {
        matches!(self, Self::DeleteBackward)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_constructor_and_accessor() {
        let edit = TextEdit::insert('a');
        assert_eq!(edit, TextEdit::Insert('a'));
        assert_eq!(edit.inserted(), Some('a'));
        assert!(!edit.is_delete_backward());
    }

    #[test]
    fn delete_backward_has_no_inserted_char() {
        let edit = TextEdit::DeleteBackward;
        assert_eq!(edit.inserted(), None);
        assert!(edit.is_delete_backward());
    }
}
