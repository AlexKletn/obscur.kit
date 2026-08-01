#![doc = include_str!("edit_transaction.doc/_module.md")]

use crate::selection::TextSelection;
use crate::span::TextRange;

#[doc = include_str!("edit_transaction.doc/EditTransaction.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct EditTransaction {
    #[doc = include_str!("edit_transaction.doc/range.md")]
    range: TextRange,
    #[doc = include_str!("edit_transaction.doc/removed.md")]
    removed: Box<str>,
    #[doc = include_str!("edit_transaction.doc/inserted.md")]
    inserted: Box<str>,
    #[doc = include_str!("edit_transaction.doc/selection_before.md")]
    selection_before: TextSelection,
    #[doc = include_str!("edit_transaction.doc/selection_after.md")]
    selection_after: TextSelection,
}

impl EditTransaction {
    #[doc = include_str!("edit_transaction.doc/new.md")]
    #[must_use]
    pub fn new(
        range: TextRange,
        removed: impl Into<Box<str>>,
        inserted: impl Into<Box<str>>,
        selection_before: TextSelection,
        selection_after: TextSelection,
    ) -> Self {
        Self {
            range,
            removed: removed.into(),
            inserted: inserted.into(),
            selection_before,
            selection_after,
        }
    }

    #[doc = include_str!("edit_transaction.doc/range_2.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    #[doc = include_str!("edit_transaction.doc/removed_2.md")]
    #[must_use]
    pub fn removed(&self) -> &str {
        &self.removed
    }

    #[doc = include_str!("edit_transaction.doc/inserted_2.md")]
    #[must_use]
    pub fn inserted(&self) -> &str {
        &self.inserted
    }

    #[doc = include_str!("edit_transaction.doc/selection_before_2.md")]
    #[must_use]
    pub const fn selection_before(&self) -> TextSelection {
        self.selection_before
    }

    #[doc = include_str!("edit_transaction.doc/selection_after_2.md")]
    #[must_use]
    pub const fn selection_after(&self) -> TextSelection {
        self.selection_after
    }

    #[doc = include_str!("edit_transaction.doc/inserted_range.md")]
    #[must_use]
    pub const fn inserted_range(&self) -> TextRange {
        TextRange::new(self.range.start, self.range.start + self.inserted.len())
    }

    #[doc = include_str!("edit_transaction.doc/invert.md")]
    #[must_use]
    pub fn invert(&self) -> EditTransaction {
        EditTransaction {
            range: self.inserted_range(),
            removed: self.inserted.clone(),
            inserted: self.removed.clone(),
            selection_before: self.selection_after,
            selection_after: self.selection_before,
        }
    }

    #[doc = include_str!("edit_transaction.doc/is_noop.md")]
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.removed == self.inserted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> EditTransaction {
        // Replace bytes 2..5 ("abc") with "XY".
        EditTransaction::new(
            TextRange::new(2, 5),
            "abc",
            "XY",
            TextSelection::caret(5),
            TextSelection::caret(4),
        )
    }

    #[test]
    fn inserted_range_is_start_plus_inserted_len() {
        assert_eq!(sample().inserted_range(), TextRange::new(2, 4));
    }

    #[test]
    fn invert_swaps_text_range_and_selections() {
        let inv = sample().invert();
        // New range is where the inserted text now sits.
        assert_eq!(inv.range(), TextRange::new(2, 4));
        assert_eq!(inv.removed(), "XY");
        assert_eq!(inv.inserted(), "abc");
        assert_eq!(inv.selection_before(), TextSelection::caret(4));
        assert_eq!(inv.selection_after(), TextSelection::caret(5));
    }

    #[test]
    fn invert_round_trips() {
        let t = sample();
        assert_eq!(t.invert().invert(), t);
    }

    #[test]
    fn is_noop_detects_unchanged_text() {
        assert!(
            EditTransaction::new(
                TextRange::new(1, 2),
                "a",
                "a",
                TextSelection::caret(1),
                TextSelection::caret(2),
            )
            .is_noop()
        );
        assert!(!sample().is_noop());
    }
}
