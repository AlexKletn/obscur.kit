#![doc = include_str!("../../docs/api/layout/white_space.doc/_module.md")]

#[doc = include_str!("../../docs/api/layout/white_space.doc/WhiteSpace.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WhiteSpace {
    /// `normal` (the initial): collapse whitespace runs to one space, collapse a
    /// segment break to a space, and allow soft wrapping.
    #[default]
    Normal,
    /// `nowrap`: collapse like `normal`, but never soft-wrap (a single line).
    Nowrap,
    /// `pre`: preserve spaces, tabs, and segment breaks (forced breaks); no soft
    /// wrapping.
    Pre,
    /// `pre-wrap`: preserve spaces, tabs, and segment breaks, and also soft-wrap.
    PreWrap,
    /// `pre-line`: collapse whitespace runs to one space, but preserve segment
    /// breaks as forced breaks, and soft-wrap.
    PreLine,
}

impl WhiteSpace {
    /// Whether runs of collapsible spaces/tabs collapse to a single space and
    /// leading/trailing whitespace is trimmed at the pre-shape seam (§4.1.1) —
    /// true for [`Normal`](Self::Normal), [`Nowrap`](Self::Nowrap), and
    /// [`PreLine`](Self::PreLine); false for the preserving [`Pre`](Self::Pre) /
    /// [`PreWrap`](Self::PreWrap).
    #[must_use]
    pub const fn collapses_spaces(self) -> bool {
        matches!(self, Self::Normal | Self::Nowrap | Self::PreLine)
    }

    /// Whether a segment break (`\n`) is preserved as a **mandatory** line break
    /// (§4.1.1) — true for [`Pre`](Self::Pre), [`PreWrap`](Self::PreWrap), and
    /// [`PreLine`](Self::PreLine); false for [`Normal`](Self::Normal) /
    /// [`Nowrap`](Self::Nowrap), where the break collapses to a space.
    #[must_use]
    pub const fn preserves_segment_breaks(self) -> bool {
        matches!(self, Self::Pre | Self::PreWrap | Self::PreLine)
    }

    /// Whether soft (optional) wrap opportunities exist for this mode (§3) — true
    /// for [`Normal`](Self::Normal), [`PreWrap`](Self::PreWrap), and
    /// [`PreLine`](Self::PreLine); false for the non-wrapping [`Nowrap`](Self::Nowrap)
    /// / [`Pre`](Self::Pre), which lay each segment on a single (possibly
    /// overflowing) line.
    #[must_use]
    pub const fn wraps(self) -> bool {
        matches!(self, Self::Normal | Self::PreWrap | Self::PreLine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(WhiteSpace::default(), WhiteSpace::Normal);
    }

    #[test]
    fn collapse_bits_match_the_spec_table() {
        assert!(WhiteSpace::Normal.collapses_spaces());
        assert!(WhiteSpace::Nowrap.collapses_spaces());
        assert!(WhiteSpace::PreLine.collapses_spaces());
        assert!(!WhiteSpace::Pre.collapses_spaces());
        assert!(!WhiteSpace::PreWrap.collapses_spaces());
    }

    #[test]
    fn segment_break_bits_match_the_spec_table() {
        assert!(WhiteSpace::Pre.preserves_segment_breaks());
        assert!(WhiteSpace::PreWrap.preserves_segment_breaks());
        assert!(WhiteSpace::PreLine.preserves_segment_breaks());
        assert!(!WhiteSpace::Normal.preserves_segment_breaks());
        assert!(!WhiteSpace::Nowrap.preserves_segment_breaks());
    }

    #[test]
    fn wrap_bits_match_the_spec_table() {
        assert!(WhiteSpace::Normal.wraps());
        assert!(WhiteSpace::PreWrap.wraps());
        assert!(WhiteSpace::PreLine.wraps());
        assert!(!WhiteSpace::Nowrap.wraps());
        assert!(!WhiteSpace::Pre.wraps());
    }
}
