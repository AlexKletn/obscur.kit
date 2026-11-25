#[doc = include_str!("../../docs/api/layout/break_mode.doc/WordBreak.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WordBreak {
    /// `normal` (the initial): ordinary word / CJK-ideograph break points only.
    #[default]
    Normal,
    /// `break-all`: a soft-wrap opportunity exists between any two typographic
    /// units, so the greedy breaker may fill each line by splitting mid-word at a
    /// grapheme-cluster boundary (CSS Text 3 §5.2).
    BreakAll,
    /// `keep-all`: no break is allowed between the letters of CJK-context text;
    /// implicit soft-wrap opportunities between two adjacent CJK ideographs are
    /// suppressed (CSS Text 3 §5.2).
    KeepAll,
}

impl WordBreak {
    /// Whether this value forces greedy any-boundary breaking — true only for
    /// [`BreakAll`](Self::BreakAll) (CSS Text 3 §5.2).
    #[must_use]
    pub const fn breaks_all(self) -> bool {
        matches!(self, Self::BreakAll)
    }

    /// Whether this value forbids breaking between adjacent CJK ideographs — true
    /// only for [`KeepAll`](Self::KeepAll) (CSS Text 3 §5.2).
    #[must_use]
    pub const fn keeps_all(self) -> bool {
        matches!(self, Self::KeepAll)
    }
}

#[cfg(test)]
mod tests {
    use super::WordBreak;

    #[test]
    fn defaults_to_normal() {
        assert_eq!(WordBreak::default(), WordBreak::Normal);
    }

    #[test]
    fn word_break_predicates() {
        assert!(WordBreak::BreakAll.breaks_all());
        assert!(!WordBreak::Normal.breaks_all());
        assert!(WordBreak::KeepAll.keeps_all());
        assert!(!WordBreak::Normal.keeps_all());
    }
}
