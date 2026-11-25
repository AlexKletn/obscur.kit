//! Line-break opportunities discovered in a run of text (spec §5.10).

use super::{BreakKind, WhiteSpace};

#[doc = include_str!("../../docs/api/layout/line_break.doc/LineBreak.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineBreak {
    /// The byte offset in the source where the break may or must occur.
    offset: usize,
    /// Whether the break is optional ([`BreakKind::Soft`]) or mandatory
    /// ([`BreakKind::Hard`]).
    kind: BreakKind,
}

impl LineBreak {
    /// Creates a [`Soft`](BreakKind::Soft) (permitted) break at byte `offset`.
    #[must_use]
    pub const fn soft(offset: usize) -> Self {
        Self {
            offset,
            kind: BreakKind::Soft,
        }
    }

    /// Creates a [`Hard`](BreakKind::Hard) (mandatory) break at byte `offset`.
    #[must_use]
    pub const fn hard(offset: usize) -> Self {
        Self {
            offset,
            kind: BreakKind::Hard,
        }
    }

    /// Returns the byte offset in the source where the break occurs.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the break's [`BreakKind`].
    #[must_use]
    pub const fn kind(&self) -> BreakKind {
        self.kind
    }

    /// Returns whether this break is mandatory ([`BreakKind::Hard`]).
    #[must_use]
    pub const fn is_mandatory(&self) -> bool {
        matches!(self.kind, BreakKind::Hard)
    }

    #[doc = include_str!("../../docs/api/layout/line_break.doc/opportunities.md")]
    #[must_use]
    pub fn opportunities(text: &str, white_space: WhiteSpace) -> Vec<Self> {
        let mut breaks = Vec::new();
        let wraps = white_space.wraps();
        let preserves_breaks = white_space.preserves_segment_breaks();
        let mut in_whitespace = false;
        let mut whitespace_had_hard_break = false;
        for (index, ch) in text.char_indices() {
            if ch == '\n' && preserves_breaks {
                breaks.push(Self::hard(index + ch.len_utf8()));
                whitespace_had_hard_break = true;
            }
            if ch.is_whitespace() {
                in_whitespace = true;
            } else {
                // Transition from a whitespace run into a word: a soft break
                // sits before this word, unless the mode disallows soft wrapping
                // or a hard break already covers it.
                if in_whitespace && wraps && !whitespace_had_hard_break {
                    breaks.push(Self::soft(index));
                }
                in_whitespace = false;
                whitespace_had_hard_break = false;
            }
        }
        breaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_and_accessors() {
        let soft = LineBreak::soft(4);
        assert_eq!(soft.offset(), 4);
        assert_eq!(soft.kind(), BreakKind::Soft);
        assert!(!soft.is_mandatory());

        let hard = LineBreak::hard(7);
        assert_eq!(hard.offset(), 7);
        assert_eq!(hard.kind(), BreakKind::Hard);
        assert!(hard.is_mandatory());
    }

    #[test]
    fn ordering_is_offset_then_kind() {
        assert!(LineBreak::soft(2) < LineBreak::hard(4));
        // Same offset: Soft sorts before Hard.
        assert!(LineBreak::soft(4) < LineBreak::hard(4));
    }

    #[test]
    fn opportunities_of_a_b_newline_c_under_preserving_breaks() {
        // "a b\nc" with a mode that preserves segment breaks (`pre-line`): soft
        // break before "b" (byte 2), hard break after "\n" (byte 4).
        let breaks = LineBreak::opportunities("a b\nc", WhiteSpace::PreLine);
        assert_eq!(breaks, vec![LineBreak::soft(2), LineBreak::hard(4)]);
    }

    #[test]
    fn newline_run_yields_only_hard_break_when_preserved() {
        // The word after a preserved newline gets no duplicate soft break.
        let breaks = LineBreak::opportunities("a\nb", WhiteSpace::PreLine);
        assert_eq!(breaks, vec![LineBreak::hard(2)]);
    }

    #[test]
    fn soft_break_after_run_of_spaces_is_utf8_byte_correct() {
        // "é  x": 'é' is two bytes (0..2), two spaces (2, 3), 'x' at byte 4.
        let breaks = LineBreak::opportunities("é  x", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(4)]);
    }

    #[test]
    fn no_opportunities_in_a_single_word() {
        assert!(LineBreak::opportunities("hello", WhiteSpace::Normal).is_empty());
    }

    #[test]
    fn normal_treats_newline_as_soft_whitespace_not_hard() {
        // Under `normal` a `\n` is NOT preserved (§3): it is ordinary whitespace,
        // so "a\nb" yields a single soft break before "b" (byte 2), no hard break.
        let breaks = LineBreak::opportunities("a\nb", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(2)]);
    }

    #[test]
    fn nowrap_yields_no_breaks_at_all() {
        // `nowrap`: no soft wrap and (segment breaks collapsed) no hard break —
        // the whole run is one line.
        assert!(LineBreak::opportunities("a b\nc", WhiteSpace::Nowrap).is_empty());
    }

    #[test]
    fn pre_yields_only_hard_breaks_no_soft() {
        // `pre`: preserve segment breaks (hard at byte 4) but no soft wrap, so the
        // space before "b" yields no soft break.
        let breaks = LineBreak::opportunities("a b\nc", WhiteSpace::Pre);
        assert_eq!(breaks, vec![LineBreak::hard(4)]);
    }

    #[test]
    fn pre_wrap_yields_both_soft_and_hard() {
        // `pre-wrap`: soft before "b" (byte 2) and hard after "\n" (byte 4).
        let breaks = LineBreak::opportunities("a b\nc", WhiteSpace::PreWrap);
        assert_eq!(breaks, vec![LineBreak::soft(2), LineBreak::hard(4)]);
    }
}
