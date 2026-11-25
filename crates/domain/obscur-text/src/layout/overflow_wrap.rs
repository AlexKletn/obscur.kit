#![doc = include_str!("../../docs/api/layout/break_mode.doc/_module.md")]

#[doc = include_str!("../../docs/api/layout/break_mode.doc/OverflowWrap.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverflowWrap {
    /// `normal` (the initial): lines may break only at ordinary break
    /// opportunities; an over-wide unbreakable word overflows (CSS Text 3 §5.2).
    #[default]
    Normal,
    /// `break-word`: an otherwise unbreakable word that does not fit on its own
    /// line may be broken at an arbitrary grapheme-cluster boundary, as a last
    /// resort, to keep it from overflowing (CSS Text 3 §5.2).
    BreakWord,
    /// `anywhere`: like [`BreakWord`](Self::BreakWord) for the greedy line
    /// breaker — a last-resort grapheme-cluster break to avoid overflow. (It also
    /// differs from `break-word` in how it affects min-content intrinsic sizing,
    /// which the greedy wrapper does not distinguish.)
    Anywhere,
}

impl OverflowWrap {
    /// Whether this value permits a *last-resort* emergency break inside an
    /// over-wide word — true for [`BreakWord`](Self::BreakWord) and
    /// [`Anywhere`](Self::Anywhere), false for [`Normal`](Self::Normal).
    #[must_use]
    pub const fn allows_emergency_break(self) -> bool {
        matches!(self, Self::BreakWord | Self::Anywhere)
    }
}

#[cfg(test)]
mod tests {
    use super::OverflowWrap;

    #[test]
    fn defaults_to_normal() {
        assert_eq!(OverflowWrap::default(), OverflowWrap::Normal);
    }

    #[test]
    fn overflow_wrap_emergency_predicate() {
        assert!(!OverflowWrap::Normal.allows_emergency_break());
        assert!(OverflowWrap::BreakWord.allows_emergency_break());
        assert!(OverflowWrap::Anywhere.allows_emergency_break());
    }
}
