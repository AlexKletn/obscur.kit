//! The `overflow-wrap` and `word-break` line-break controls (CSS Text 3 §5.2).
//!
//! These two computed values govern whether — and how aggressively — the line
//! breaker may split *within* an otherwise unbreakable token so it does not
//! overflow its line. They are the emergency-break companions of the ordinary
//! (whitespace / UAX#14) break opportunities: the break opportunities say where a
//! line *may* break between words, while these say whether a word that still does
//! not fit may be forced apart at a grapheme-cluster boundary.
//!
//! Both properties inherit and both default to their `normal` value, which keeps
//! the historical behaviour exactly: an over-wide unbreakable token overflows its
//! line rather than being split (CSS Text 3 §5.2, which is why an English word
//! wider than its column pokes out of the box by default).

/// The computed `overflow-wrap` value (CSS Text 3 §5.2; the legacy alias
/// `word-wrap` maps onto it).
///
/// `overflow-wrap` permits a *last-resort* break inside an otherwise unbreakable
/// run of text, but **only** when the word would not otherwise fit on a line by
/// itself — i.e. to avoid content overflowing its line box. It never breaks a
/// word that already fits on its own line.
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

/// The computed `word-break` value (CSS Text 3 §5.2).
///
/// `word-break` selects the *soft-wrap* granularity within a run of text.
/// [`Normal`](Self::Normal) uses the ordinary rules (break between words, and —
/// via the default breaker — between adjacent CJK ideographs).
/// [`BreakAll`](Self::BreakAll) permits a break between any two typographic
/// units, so a long word wraps greedily mid-word. [`KeepAll`](Self::KeepAll)
/// forbids breaking between the letters of CJK-context text (its main observable
/// effect versus `normal`).
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
    use super::*;

    #[test]
    fn defaults_are_normal() {
        assert_eq!(OverflowWrap::default(), OverflowWrap::Normal);
        assert_eq!(WordBreak::default(), WordBreak::Normal);
    }

    #[test]
    fn overflow_wrap_emergency_predicate() {
        assert!(!OverflowWrap::Normal.allows_emergency_break());
        assert!(OverflowWrap::BreakWord.allows_emergency_break());
        assert!(OverflowWrap::Anywhere.allows_emergency_break());
    }

    #[test]
    fn word_break_predicates() {
        assert!(WordBreak::BreakAll.breaks_all());
        assert!(!WordBreak::Normal.breaks_all());
        assert!(WordBreak::KeepAll.keeps_all());
        assert!(!WordBreak::Normal.keeps_all());
    }
}
