//! The granularity of a delete operation in the editing engine (spec §5.11).

/// The granularity of a deletion, i.e. how much text one delete step removes
/// (spec §5.11 editing engine — deletion semantics).
///
/// A [`DeletionUnit`] answers "how much?" for a delete; it pairs with an
/// [`EditDirection`](crate::editing::EditDirection) in
/// [`EditCommand::Delete`](crate::editing::EditCommand::Delete) so the same unit
/// covers both the backward (Backspace) and forward (Delete) forms. When the
/// selection is non-empty the engine deletes the selection instead and the unit
/// is ignored; the unit governs the collapsed-caret case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeletionUnit {
    /// One extended grapheme cluster — Backspace (backward) / Delete (forward).
    Grapheme,
    /// One word — Ctrl+Backspace (backward) / Ctrl+Delete (forward).
    Word,
    /// The whole line the caret sits on.
    Line,
    /// From the caret to the start of its line — e.g. Cmd+Backspace on macOS.
    ToLineStart,
    /// From the caret to the end of its line — e.g. Ctrl+K.
    ToLineEnd,
    /// From the caret to the start of the document.
    ToDocumentStart,
    /// From the caret to the end of the document.
    ToDocumentEnd,
}

impl DeletionUnit {
    /// Returns whether this unit is resolved relative to the caret's line — the
    /// engine needs the current line's extent to compute the deleted range.
    ///
    /// True for [`Line`](Self::Line), [`ToLineStart`](Self::ToLineStart), and
    /// [`ToLineEnd`](Self::ToLineEnd).
    #[must_use]
    pub const fn is_line_relative(&self) -> bool {
        matches!(self, Self::Line | Self::ToLineStart | Self::ToLineEnd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_relative_units() {
        assert!(DeletionUnit::Line.is_line_relative());
        assert!(DeletionUnit::ToLineStart.is_line_relative());
        assert!(DeletionUnit::ToLineEnd.is_line_relative());
    }

    #[test]
    fn non_line_relative_units() {
        assert!(!DeletionUnit::Grapheme.is_line_relative());
        assert!(!DeletionUnit::Word.is_line_relative());
        assert!(!DeletionUnit::ToDocumentStart.is_line_relative());
        assert!(!DeletionUnit::ToDocumentEnd.is_line_relative());
    }
}
