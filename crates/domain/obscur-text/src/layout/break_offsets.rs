#![doc = include_str!("break_offsets.doc/_module.md")]

use std::collections::HashSet;

use super::{Hyphenator, LineBreak, WhiteSpace};

#[doc = include_str!("break_offsets.doc/BreakOffsets.md")]
pub(super) struct BreakOffsets {
    #[doc = include_str!("break_offsets.doc/hyphens.md")]
    hyphens: HashSet<usize>,
    #[doc = include_str!("break_offsets.doc/soft.md")]
    soft: HashSet<usize>,
    #[doc = include_str!("break_offsets.doc/hangable.md")]
    hangable: HashSet<usize>,
}

impl BreakOffsets {
    #[doc = include_str!("break_offsets.doc/collect.md")]
    pub(super) fn collect(
        text: &str,
        breaks: &[LineBreak],
        hyphenator: &dyn Hyphenator,
        white_space: WhiteSpace,
    ) -> Self {
        let soft: HashSet<usize> = text
            .char_indices()
            .filter(|&(_, ch)| ch == '\u{00AD}')
            .map(|(index, _)| index)
            .collect();
        let hangable: HashSet<usize> = if white_space.collapses_spaces() {
            text.char_indices()
                .filter(|&(_, ch)| ch == ' ' || ch == '\t')
                .map(|(index, _)| index)
                .collect()
        } else {
            HashSet::new()
        };
        Self {
            hyphens: Self::hyphen_offsets(text, breaks, hyphenator),
            soft,
            hangable,
        }
    }

    #[doc = include_str!("break_offsets.doc/is_hyphen_point.md")]
    pub(super) fn is_hyphen_point(&self, cluster: usize) -> bool {
        self.hyphens.contains(&cluster)
    }

    #[doc = include_str!("break_offsets.doc/is_soft_hyphen.md")]
    pub(super) fn is_soft_hyphen(&self, cluster: usize) -> bool {
        self.soft.contains(&cluster)
    }

    #[doc = include_str!("break_offsets.doc/has_soft_hyphens.md")]
    pub(super) fn has_soft_hyphens(&self) -> bool {
        !self.soft.is_empty()
    }

    #[doc = include_str!("break_offsets.doc/hangs.md")]
    pub(super) fn hangs(&self, cluster: usize) -> bool {
        self.hangable.contains(&cluster)
    }

    #[doc = include_str!("break_offsets.doc/hyphen_offsets.md")]
    fn hyphen_offsets(
        text: &str,
        breaks: &[LineBreak],
        hyphenator: &dyn Hyphenator,
    ) -> HashSet<usize> {
        let mut offsets = HashSet::new();
        let mut bounds: Vec<usize> = breaks.iter().map(LineBreak::offset).collect();
        bounds.push(text.len());
        let mut start = 0;
        for end in bounds {
            if end > start && end <= text.len() {
                let word = &text[start..end];
                for offset in hyphenator.hyphenate(word) {
                    let full = start + offset;
                    // Keep only strictly interior break points.
                    if full > start && full < end {
                        offsets.insert(full);
                    }
                }
            }
            start = end.max(start);
        }
        offsets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::layout::NullHyphenator;

    /// A hyphenator that proposes `offsets` inside every word it is offered,
    /// including out-of-range ones, so the interior filter can be observed.
    struct StubHyphenator {
        offsets: Vec<usize>,
    }

    impl Hyphenator for StubHyphenator {
        fn hyphenate(&self, word: &str) -> Vec<usize> {
            self.offsets
                .iter()
                .copied()
                .filter(|&offset| offset <= word.len())
                .collect()
        }
    }

    #[test]
    fn soft_hyphens_are_collected_by_byte_offset() {
        let text = "a\u{00AD}bc";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        assert!(offsets.has_soft_hyphens());
        assert!(offsets.is_soft_hyphen(1));
        assert!(!offsets.is_soft_hyphen(0));
    }

    #[test]
    fn hangable_spaces_exist_only_in_a_collapsing_mode() {
        let text = "a b\tc";
        let collapsing = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        assert!(collapsing.hangs(1));
        assert!(collapsing.hangs(3));
        assert!(!collapsing.hangs(0));

        // `pre-wrap` preserves its spaces, so nothing hangs and the measurement
        // of a preserved space is untouched.
        let preserving = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::PreWrap);
        assert!(!preserving.hangs(1));
        assert!(!preserving.hangs(3));
    }

    #[test]
    fn a_hyphenation_offset_at_a_word_edge_is_dropped() {
        // Words are "ab " and "cd": a proposal at 0 (the head edge) and one at
        // the word's end are not strictly interior, so only offset 1 survives.
        let text = "ab cd";
        let breaks = vec![LineBreak::soft(3)];
        let offsets = BreakOffsets::collect(
            text,
            &breaks,
            &StubHyphenator {
                offsets: vec![0, 1, 3],
            },
            WhiteSpace::Normal,
        );
        assert!(offsets.is_hyphen_point(1));
        assert!(!offsets.is_hyphen_point(0));
        assert!(!offsets.is_hyphen_point(3));
        // The second word "cd" is offered 0 and 1; only its interior 1 (full
        // offset 4) is kept.
        assert!(offsets.is_hyphen_point(4));
    }

    #[test]
    fn the_null_hyphenator_proposes_nothing() {
        let text = "hyphenation";
        let offsets = BreakOffsets::collect(text, &[], &NullHyphenator, WhiteSpace::Normal);
        assert!((0..text.len()).all(|index| !offsets.is_hyphen_point(index)));
    }
}
