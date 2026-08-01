#![doc = include_str!("text_selection.doc/_module.md")]

use crate::span::TextRange;

use super::{Caret, CaretAffinity};

#[doc = include_str!("text_selection.doc/TextSelection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSelection {
    #[doc = include_str!("text_selection.doc/anchor.md")]
    anchor: usize,
    #[doc = include_str!("text_selection.doc/focus.md")]
    focus: usize,
    #[doc = include_str!("text_selection.doc/affinity.md")]
    affinity: CaretAffinity,
}

impl TextSelection {
    #[doc = include_str!("text_selection.doc/caret.md")]
    #[must_use]
    pub const fn caret(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    #[doc = include_str!("text_selection.doc/new.md")]
    #[must_use]
    pub const fn new(anchor: usize, focus: usize) -> Self {
        Self {
            anchor,
            focus,
            affinity: CaretAffinity::Downstream,
        }
    }

    #[doc = include_str!("text_selection.doc/with_affinity.md")]
    #[must_use]
    pub const fn with_affinity(&self, affinity: CaretAffinity) -> Self {
        Self {
            anchor: self.anchor,
            focus: self.focus,
            affinity,
        }
    }

    #[doc = include_str!("text_selection.doc/anchor_2.md")]
    #[must_use]
    pub const fn anchor(&self) -> usize {
        self.anchor
    }

    #[doc = include_str!("text_selection.doc/focus_2.md")]
    #[must_use]
    pub const fn focus(&self) -> usize {
        self.focus
    }

    #[doc = include_str!("text_selection.doc/affinity_2.md")]
    #[must_use]
    pub const fn affinity(&self) -> CaretAffinity {
        self.affinity
    }

    #[doc = include_str!("text_selection.doc/range.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        TextRange::new(self.start(), self.end())
    }

    #[doc = include_str!("text_selection.doc/is_collapsed.md")]
    #[must_use]
    pub const fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    #[doc = include_str!("text_selection.doc/is_forward.md")]
    #[must_use]
    pub const fn is_forward(&self) -> bool {
        self.focus >= self.anchor
    }

    #[doc = include_str!("text_selection.doc/start.md")]
    #[must_use]
    pub const fn start(&self) -> usize {
        if self.anchor < self.focus {
            self.anchor
        } else {
            self.focus
        }
    }

    #[doc = include_str!("text_selection.doc/end.md")]
    #[must_use]
    pub const fn end(&self) -> usize {
        if self.anchor > self.focus {
            self.anchor
        } else {
            self.focus
        }
    }

    #[doc = include_str!("text_selection.doc/len.md")]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end() - self.start()
    }

    #[doc = include_str!("text_selection.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[doc = include_str!("text_selection.doc/focus_caret.md")]
    #[must_use]
    pub const fn focus_caret(&self) -> Caret {
        Caret::new(self.focus, self.affinity)
    }

    #[doc = include_str!("text_selection.doc/anchor_caret.md")]
    #[must_use]
    pub const fn anchor_caret(&self) -> Caret {
        Caret::at(self.anchor)
    }

    #[doc = include_str!("text_selection.doc/collapse_to_focus.md")]
    #[must_use]
    pub const fn collapse_to_focus(&self) -> Self {
        Self {
            anchor: self.focus,
            focus: self.focus,
            affinity: self.affinity,
        }
    }

    #[doc = include_str!("text_selection.doc/collapse_to_start.md")]
    #[must_use]
    pub const fn collapse_to_start(&self) -> Self {
        Self::caret(self.start())
    }

    #[doc = include_str!("text_selection.doc/collapse_to_end.md")]
    #[must_use]
    pub const fn collapse_to_end(&self) -> Self {
        Self::caret(self.end())
    }

    #[doc = include_str!("text_selection.doc/extend_to.md")]
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
