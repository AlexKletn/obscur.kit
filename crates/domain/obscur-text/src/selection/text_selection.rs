//! A directional anchor/focus text selection (spec §5.11).

use crate::span::TextRange;

use super::{Caret, CaretAffinity};

/// A single directional selection over a UTF-8 text buffer (spec §5.11 editing
/// engine).
///
/// A selection is defined by two *byte* offsets: the [`anchor`](Self::anchor),
/// the fixed end that stays put while the selection is extended, and the
/// [`focus`](Self::focus), the moving end that follows the caret. The order of
/// the two encodes direction — a forward selection has `focus >= anchor`, a
/// backward one has `focus < anchor` — which drives shift-arrow extension and
/// caret placement. A [`CaretAffinity`] disambiguates the focus when it lands on
/// a soft line-wrap boundary. When `anchor == focus` the selection is
/// *collapsed* and behaves as a plain caret.
///
/// This type models exactly one selection. Multi-selection (several disjoint
/// ranges edited together) is a set of `TextSelection`s owned and reconciled by
/// the buffer/editing engine, not represented here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSelection {
    /// The fixed end of the selection (byte offset).
    anchor: usize,
    /// The moving end of the selection (byte offset).
    focus: usize,
    /// The wrap-boundary affinity of the focus.
    affinity: CaretAffinity,
}

impl TextSelection {
    /// Creates a collapsed selection at `offset` (a caret): `anchor == focus`,
    /// with the default [`Downstream`](CaretAffinity::Downstream) affinity.
    #[must_use]
    pub const fn caret(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    /// Creates a selection from `anchor` to `focus` with the default
    /// [`Downstream`](CaretAffinity::Downstream) affinity.
    #[must_use]
    pub const fn new(anchor: usize, focus: usize) -> Self {
        Self {
            anchor,
            focus,
            affinity: CaretAffinity::Downstream,
        }
    }

    /// Returns a copy of this selection with the focus `affinity` replaced.
    #[must_use]
    pub const fn with_affinity(&self, affinity: CaretAffinity) -> Self {
        Self {
            anchor: self.anchor,
            focus: self.focus,
            affinity,
        }
    }

    /// Returns the fixed end of the selection (byte offset).
    #[must_use]
    pub const fn anchor(&self) -> usize {
        self.anchor
    }

    /// Returns the moving end of the selection (byte offset).
    #[must_use]
    pub const fn focus(&self) -> usize {
        self.focus
    }

    /// Returns the focus's wrap-boundary affinity.
    #[must_use]
    pub const fn affinity(&self) -> CaretAffinity {
        self.affinity
    }

    /// Returns the covered [`TextRange`], `min(anchor, focus)..max(anchor,
    /// focus)`, regardless of selection direction.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        TextRange::new(self.start(), self.end())
    }

    /// Returns whether the selection is collapsed (`anchor == focus`), i.e. a
    /// plain caret with no selected span.
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    /// Returns whether the selection points forward (`focus >= anchor`). A
    /// collapsed selection is considered forward.
    #[must_use]
    pub const fn is_forward(&self) -> bool {
        self.focus >= self.anchor
    }

    /// Returns the lower of the two endpoints (`min(anchor, focus)`).
    #[must_use]
    pub const fn start(&self) -> usize {
        if self.anchor < self.focus {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Returns the higher of the two endpoints (`max(anchor, focus)`).
    #[must_use]
    pub const fn end(&self) -> usize {
        if self.anchor > self.focus {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Returns the length of the selected span in bytes (`end - start`).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end() - self.start()
    }

    /// Returns whether the selection is empty (a collapsed caret).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the [`Caret`] at the focus, carrying the selection's affinity.
    #[must_use]
    pub const fn focus_caret(&self) -> Caret {
        Caret::new(self.focus, self.affinity)
    }

    /// Returns the [`Caret`] at the anchor, with the default
    /// [`Downstream`](CaretAffinity::Downstream) affinity.
    #[must_use]
    pub const fn anchor_caret(&self) -> Caret {
        Caret::at(self.anchor)
    }

    /// Returns a collapsed selection at the focus, keeping the affinity.
    #[must_use]
    pub const fn collapse_to_focus(&self) -> Self {
        Self {
            anchor: self.focus,
            focus: self.focus,
            affinity: self.affinity,
        }
    }

    /// Returns a collapsed selection at the start (lower endpoint).
    #[must_use]
    pub const fn collapse_to_start(&self) -> Self {
        Self::caret(self.start())
    }

    /// Returns a collapsed selection at the end (higher endpoint).
    #[must_use]
    pub const fn collapse_to_end(&self) -> Self {
        Self::caret(self.end())
    }

    /// Returns a selection with the anchor unchanged and the focus moved to
    /// `focus`, keeping the affinity. This is the shift-extend operation.
    #[must_use]
    pub const fn extend_to(&self, focus: usize) -> Self {
        Self {
            anchor: self.anchor,
            focus,
            affinity: self.affinity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_is_collapsed() {
        let sel = TextSelection::caret(4);
        assert!(sel.is_collapsed());
        assert_eq!(sel.anchor(), 4);
        assert_eq!(sel.focus(), 4);
        assert_eq!(sel.len(), 0);
        assert_eq!(sel.affinity(), CaretAffinity::Downstream);
    }

    #[test]
    fn range_normalizes_regardless_of_direction() {
        assert_eq!(TextSelection::new(2, 7).range(), TextRange::new(2, 7));
        // Backward selection yields the same range.
        assert_eq!(TextSelection::new(7, 2).range(), TextRange::new(2, 7));
        assert_eq!(TextSelection::new(7, 2).len(), 5);
    }

    #[test]
    fn is_forward_encodes_direction() {
        assert!(TextSelection::new(2, 7).is_forward());
        assert!(!TextSelection::new(7, 2).is_forward());
        // Collapsed counts as forward.
        assert!(TextSelection::caret(3).is_forward());
    }

    #[test]
    fn start_and_end() {
        let back = TextSelection::new(7, 2);
        assert_eq!(back.start(), 2);
        assert_eq!(back.end(), 7);
    }

    #[test]
    fn focus_and_anchor_carets() {
        let sel = TextSelection::new(2, 7).with_affinity(CaretAffinity::Upstream);
        assert_eq!(sel.focus_caret(), Caret::new(7, CaretAffinity::Upstream));
        assert_eq!(sel.anchor_caret(), Caret::at(2));
    }

    #[test]
    fn collapse_variants() {
        let sel = TextSelection::new(7, 2).with_affinity(CaretAffinity::Upstream);
        // collapse_to_focus keeps affinity and lands on focus (2).
        assert_eq!(
            sel.collapse_to_focus(),
            TextSelection::caret(2).with_affinity(CaretAffinity::Upstream)
        );
        // collapse_to_start/end reset to the default affinity.
        assert_eq!(sel.collapse_to_start(), TextSelection::caret(2));
        assert_eq!(sel.collapse_to_end(), TextSelection::caret(7));
    }

    #[test]
    fn extend_to_keeps_anchor() {
        let sel = TextSelection::new(2, 5).with_affinity(CaretAffinity::Upstream);
        let extended = sel.extend_to(9);
        assert_eq!(extended.anchor(), 2);
        assert_eq!(extended.focus(), 9);
        assert_eq!(extended.affinity(), CaretAffinity::Upstream);
        // Extending across the anchor flips direction.
        assert!(!sel.extend_to(0).is_forward());
    }
}
