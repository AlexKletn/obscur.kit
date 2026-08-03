#![doc = include_str!("line_break.doc/_module.md")]

use super::WhiteSpace;

#[doc = include_str!("line_break.doc/BreakKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BreakKind {
    #[doc = include_str!("line_break.doc/Soft.md")]
    Soft,
    #[doc = include_str!("line_break.doc/Hard.md")]
    Hard,
}

#[doc = include_str!("line_break.doc/LineBreak.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineBreak {
    #[doc = include_str!("line_break.doc/offset.md")]
    offset: usize,
    #[doc = include_str!("line_break.doc/kind.md")]
    kind: BreakKind,
}

impl LineBreak {
    #[doc = include_str!("line_break.doc/soft.md")]
    #[must_use]
    pub const fn soft(offset: usize) -> Self {
        Self {
            offset,
            kind: BreakKind::Soft,
        }
    }

    #[doc = include_str!("line_break.doc/hard.md")]
    #[must_use]
    pub const fn hard(offset: usize) -> Self {
        Self {
            offset,
            kind: BreakKind::Hard,
        }
    }

    #[doc = include_str!("line_break.doc/offset_2.md")]
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    #[doc = include_str!("line_break.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> BreakKind {
        self.kind
    }

    #[doc = include_str!("line_break.doc/is_mandatory.md")]
    #[must_use]
    pub const fn is_mandatory(&self) -> bool {
        matches!(self.kind, BreakKind::Hard)
    }

    #[doc = include_str!("line_break.doc/opportunities.md")]
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

#[doc = include_str!("line_break.doc/is_cjk_ideograph.md")]
#[must_use]
pub(crate) fn is_cjk_ideograph(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF      // CJK Unified Ideographs Extension A
        | 0x4E00..=0x9FFF    // CJK Unified Ideographs
        | 0xF900..=0xFAFF    // CJK Compatibility Ideographs
        | 0x2_0000..=0x2_A6DF // Extension B
        | 0x2_A700..=0x2_EBEF // Extensions C–F
        | 0x2_F800..=0x2_FA1F // CJK Compatibility Ideographs Supplement
        | 0x3_0000..=0x3_134F // Extension G
    )
}

#[doc = include_str!("line_break.doc/LineBreaker.md")]
pub trait LineBreaker {
    #[doc = include_str!("line_break.doc/opportunities_2.md")]
    fn opportunities(&self, text: &str, white_space: WhiteSpace) -> Vec<LineBreak>;
}

#[doc = include_str!("line_break.doc/DefaultLineBreaker.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefaultLineBreaker;

impl LineBreaker for DefaultLineBreaker {
    fn opportunities(&self, text: &str, white_space: WhiteSpace) -> Vec<LineBreak> {
        let mut breaks = LineBreak::opportunities(text, white_space);
        // A break opportunity sits between two adjacent CJK ideographs, but only
        // where soft wrapping is permitted (§3). This is additive: for text with
        // no ideograph pair the loop pushes nothing and `breaks` is returned
        // untouched (byte-identical to `LineBreak::opportunities`).
        if white_space.wraps() {
            let before = breaks.len();
            let mut prev: Option<char> = None;
            for (index, ch) in text.char_indices() {
                if let Some(prev_ch) = prev
                    && is_cjk_ideograph(prev_ch)
                    && is_cjk_ideograph(ch)
                {
                    breaks.push(LineBreak::soft(index));
                }
                prev = Some(ch);
            }
            if breaks.len() != before {
                breaks.sort_unstable();
                breaks.dedup();
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

    #[test]
    fn default_breaker_reproduces_opportunities_for_non_cjk_text() {
        // Requirement (d): the port default is byte-identical to
        // `LineBreak::opportunities` for every whitespace mode on Latin text.
        for text in ["hello", "a b\nc", "aa bb cc", "é  x", "ab"] {
            for mode in [
                WhiteSpace::Normal,
                WhiteSpace::Nowrap,
                WhiteSpace::Pre,
                WhiteSpace::PreWrap,
                WhiteSpace::PreLine,
            ] {
                assert_eq!(
                    DefaultLineBreaker.opportunities(text, mode),
                    LineBreak::opportunities(text, mode),
                    "{text:?} / {mode:?}"
                );
            }
        }
    }

    #[test]
    fn default_breaker_adds_soft_break_between_adjacent_cjk_ideographs() {
        // "中文字" — three CJK ideographs (3 bytes each): soft breaks before the
        // 2nd (byte 3) and 3rd (byte 6) so unspaced CJK wraps out of the box.
        let breaks = DefaultLineBreaker.opportunities("中文字", WhiteSpace::Normal);
        assert_eq!(breaks, vec![LineBreak::soft(3), LineBreak::soft(6)]);
        // A non-wrapping mode (`nowrap`) yields no CJK soft breaks.
        assert!(
            DefaultLineBreaker
                .opportunities("中文字", WhiteSpace::Nowrap)
                .is_empty()
        );
    }

    #[test]
    fn default_breaker_is_object_safe() {
        let breaker: &dyn LineBreaker = &DefaultLineBreaker;
        assert!(
            breaker
                .opportunities("hello", WhiteSpace::Normal)
                .is_empty()
        );
    }

    #[test]
    fn cjk_ideograph_range() {
        assert!(is_cjk_ideograph('中'));
        assert!(is_cjk_ideograph('字'));
        assert!(!is_cjk_ideograph('a'));
        assert!(!is_cjk_ideograph('Я'));
        // Hiragana is deliberately excluded from the simplified default set.
        assert!(!is_cjk_ideograph('あ'));
    }
}
