#![doc = include_str!("deletion_unit.doc/_module.md")]

#[doc = include_str!("deletion_unit.doc/DeletionUnit.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeletionUnit {
    #[doc = include_str!("deletion_unit.doc/Grapheme.md")]
    Grapheme,
    #[doc = include_str!("deletion_unit.doc/Word.md")]
    Word,
    #[doc = include_str!("deletion_unit.doc/Line.md")]
    Line,
    #[doc = include_str!("deletion_unit.doc/ToLineStart.md")]
    ToLineStart,
    #[doc = include_str!("deletion_unit.doc/ToLineEnd.md")]
    ToLineEnd,
    #[doc = include_str!("deletion_unit.doc/ToDocumentStart.md")]
    ToDocumentStart,
    #[doc = include_str!("deletion_unit.doc/ToDocumentEnd.md")]
    ToDocumentEnd,
}

impl DeletionUnit {
    #[doc = include_str!("deletion_unit.doc/is_line_relative.md")]
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
