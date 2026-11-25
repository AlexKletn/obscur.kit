//! The granularity of a delete operation in the editing engine (spec §5.11).

#[doc = include_str!("../../docs/api/editing/deletion_unit.doc/DeletionUnit.md")]
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
    #[doc = include_str!("../../docs/api/editing/deletion_unit.doc/is_line_relative.md")]
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
