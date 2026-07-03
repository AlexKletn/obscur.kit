//! The vocabulary of editing operations submitted to the engine (spec §5.11).

use crate::selection::TextSelection;
use crate::span::TextRange;

use super::{DeletionUnit, NavigationUnit};

/// The direction an edit operation travels relative to the caret (spec §5.11).
///
/// Distinguishes the two halves of the symmetric editing keys — Backspace vs
/// Delete, Left vs Right — so a single [`DeletionUnit`] or [`NavigationUnit`]
/// serves both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditDirection {
    /// Toward the start of the buffer (left / up / Backspace).
    Backward,
    /// Toward the end of the buffer (right / down / Delete).
    Forward,
}

/// A single editing operation submitted to the editing engine (spec §5.11
/// editing engine).
///
/// [`EditCommand`] is the engine's input vocabulary: a platform keymap resolves
/// key chords, clipboard actions, and IME events into these commands, which the
/// engine applies to the buffer and selection. The command names *what* to do;
/// resolving it against the current text and selection — and producing the
/// invertible [`EditTransaction`](crate::editing::EditTransaction) — is the
/// engine's job, not the command's.
#[derive(Debug, Clone, PartialEq)]
pub enum EditCommand {
    /// Insert text at the caret, replacing the current selection if it is
    /// non-empty (typing, IME commit of simple text, or programmatic insert).
    Insert(Box<str>),
    /// Delete `unit` worth of text in `direction` from the caret, or the
    /// selection if one is active (deletion semantics).
    Delete {
        /// How much text to remove.
        unit: DeletionUnit,
        /// Which side of the caret to remove it from.
        direction: EditDirection,
    },
    /// Move the caret by one `unit` in `direction`. When `extend` is true the
    /// selection's anchor is kept and only the focus moves (shift-extend);
    /// otherwise the selection collapses to the moved caret.
    Move {
        /// The granularity of the movement.
        unit: NavigationUnit,
        /// The direction to move.
        direction: EditDirection,
        /// Whether to extend the selection (shift held) rather than collapse it.
        extend: bool,
    },
    /// Select the entire document (Ctrl+A / Cmd+A).
    SelectAll,
    /// Replace the current selection with `selection` outright — used to seat a
    /// caret from a pointer hit or to restore a saved selection.
    SetSelection(TextSelection),
    /// Replace an explicit byte `range` with `text`, independent of the current
    /// caret — used for IME commits over the composition span and for
    /// programmatic edits.
    Replace {
        /// The pre-edit byte range to replace.
        range: TextRange,
        /// The replacement text.
        text: Box<str>,
    },
}

impl EditCommand {
    /// Creates an [`Insert`](Self::Insert) command from anything convertible into
    /// a `Box<str>` (`&str`, `String`, `Box<str>`).
    #[must_use]
    pub fn insert(text: impl Into<Box<str>>) -> Self {
        Self::Insert(text.into())
    }

    /// Returns whether applying this command mutates the buffer's text.
    ///
    /// [`Insert`](Self::Insert), [`Delete`](Self::Delete), and
    /// [`Replace`](Self::Replace) change text and therefore produce an
    /// [`EditTransaction`](crate::editing::EditTransaction);
    /// [`Move`](Self::Move), [`SelectAll`](Self::SelectAll), and
    /// [`SetSelection`](Self::SetSelection) only move the selection.
    #[must_use]
    pub const fn is_mutating(&self) -> bool {
        matches!(
            self,
            Self::Insert(_) | Self::Delete { .. } | Self::Replace { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_helper_accepts_str_and_string() {
        assert_eq!(EditCommand::insert("hi"), EditCommand::Insert("hi".into()));
        assert_eq!(
            EditCommand::insert(String::from("hi")),
            EditCommand::Insert("hi".into())
        );
    }

    #[test]
    fn mutating_commands() {
        assert!(EditCommand::insert("x").is_mutating());
        assert!(EditCommand::Delete {
            unit: DeletionUnit::Grapheme,
            direction: EditDirection::Backward,
        }
        .is_mutating());
        assert!(EditCommand::Replace {
            range: TextRange::new(0, 2),
            text: "z".into(),
        }
        .is_mutating());
    }

    #[test]
    fn non_mutating_commands() {
        assert!(!EditCommand::Move {
            unit: NavigationUnit::Word,
            direction: EditDirection::Forward,
            extend: true,
        }
        .is_mutating());
        assert!(!EditCommand::SelectAll.is_mutating());
        assert!(!EditCommand::SetSelection(TextSelection::caret(3)).is_mutating());
    }
}
