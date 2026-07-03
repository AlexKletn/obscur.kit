//! An atomic, invertible record of one text change (spec §5.11).

use crate::selection::TextSelection;
use crate::span::TextRange;

/// An atomic, invertible record of one text replacement for local undo/redo
/// (spec §5.11 editing engine — local undo/redo primitives).
///
/// A transaction captures a single splice: the byte [`range`](Self::range) in
/// the *pre-edit* text that was replaced, the [`removed`](Self::removed) text
/// that occupied it, and the [`inserted`](Self::inserted) text put in its place,
/// plus the selection immediately before and after. Because it stores both sides
/// of the splice it is fully invertible ([`invert`](Self::invert)) with no access
/// to the buffer, which is the basis of the undo stack: undo applies the
/// inverse, redo re-applies the original.
///
/// Pure insertion is `removed == ""` with an empty `range`; pure deletion is
/// `inserted == ""`.
#[derive(Debug, Clone, PartialEq)]
pub struct EditTransaction {
    /// The pre-edit byte range that was replaced.
    range: TextRange,
    /// The exact text that occupied `range` before the edit.
    removed: Box<str>,
    /// The text that replaced `removed`.
    inserted: Box<str>,
    /// The selection immediately before the edit was applied.
    selection_before: TextSelection,
    /// The selection immediately after the edit was applied.
    selection_after: TextSelection,
}

impl EditTransaction {
    /// Creates a transaction recording that `range` (in the pre-edit text), which
    /// held `removed`, was replaced by `inserted`, moving the selection from
    /// `selection_before` to `selection_after`.
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

    /// Returns the pre-edit byte range that was replaced.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the text that was removed by the edit.
    #[must_use]
    pub fn removed(&self) -> &str {
        &self.removed
    }

    /// Returns the text that was inserted by the edit.
    #[must_use]
    pub fn inserted(&self) -> &str {
        &self.inserted
    }

    /// Returns the selection recorded before the edit.
    #[must_use]
    pub const fn selection_before(&self) -> TextSelection {
        self.selection_before
    }

    /// Returns the selection recorded after the edit.
    #[must_use]
    pub const fn selection_after(&self) -> TextSelection {
        self.selection_after
    }

    /// Returns the byte range that the inserted text occupies in the *post-edit*
    /// text: `range.start .. range.start + inserted.len()`.
    ///
    /// The splice replaces `range` (length `removed.len()`) with `inserted`, so
    /// the inserted bytes begin where the removed bytes began (`range.start`) and
    /// span `inserted.len()` bytes; text after the splice shifts by
    /// `inserted.len() - removed.len()`.
    #[must_use]
    pub const fn inserted_range(&self) -> TextRange {
        TextRange::new(self.range.start, self.range.start + self.inserted.len())
    }

    /// Returns the inverse edit: applying it to the post-edit text restores the
    /// pre-edit state.
    ///
    /// The inverse replaces the [`inserted_range`](Self::inserted_range) — where
    /// the inserted text now sits, `range.start .. range.start + inserted.len()`
    /// — putting `inserted` and `removed` back in swapped roles, and swaps the
    /// before/after selections. Inverting twice is the identity for a real edit.
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

    /// Returns whether the edit changes no text (`removed == inserted`), so
    /// applying or inverting it leaves the buffer untouched.
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
        assert!(EditTransaction::new(
            TextRange::new(1, 2),
            "a",
            "a",
            TextSelection::caret(1),
            TextSelection::caret(2),
        )
        .is_noop());
        assert!(!sample().is_noop());
    }
}
