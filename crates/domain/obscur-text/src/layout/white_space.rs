#![doc = include_str!("white_space.doc/_module.md")]

#[doc = include_str!("white_space.doc/WhiteSpace.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WhiteSpace {
    #[doc = include_str!("white_space.doc/Normal.md")]
    #[default]
    Normal,
    #[doc = include_str!("white_space.doc/Nowrap.md")]
    Nowrap,
    #[doc = include_str!("white_space.doc/Pre.md")]
    Pre,
    #[doc = include_str!("white_space.doc/PreWrap.md")]
    PreWrap,
    #[doc = include_str!("white_space.doc/PreLine.md")]
    PreLine,
}

impl WhiteSpace {
    #[doc = include_str!("white_space.doc/collapses_spaces.md")]
    #[must_use]
    pub const fn collapses_spaces(self) -> bool {
        matches!(self, Self::Normal | Self::Nowrap | Self::PreLine)
    }

    #[doc = include_str!("white_space.doc/preserves_segment_breaks.md")]
    #[must_use]
    pub const fn preserves_segment_breaks(self) -> bool {
        matches!(self, Self::Pre | Self::PreWrap | Self::PreLine)
    }

    #[doc = include_str!("white_space.doc/wraps.md")]
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
        // collapse spaces: normal / nowrap / pre-line.
        assert!(WhiteSpace::Normal.collapses_spaces());
        assert!(WhiteSpace::Nowrap.collapses_spaces());
        assert!(WhiteSpace::PreLine.collapses_spaces());
        assert!(!WhiteSpace::Pre.collapses_spaces());
        assert!(!WhiteSpace::PreWrap.collapses_spaces());
    }

    #[test]
    fn segment_break_bits_match_the_spec_table() {
        // preserve `\n` as a forced break: pre / pre-wrap / pre-line.
        assert!(WhiteSpace::Pre.preserves_segment_breaks());
        assert!(WhiteSpace::PreWrap.preserves_segment_breaks());
        assert!(WhiteSpace::PreLine.preserves_segment_breaks());
        assert!(!WhiteSpace::Normal.preserves_segment_breaks());
        assert!(!WhiteSpace::Nowrap.preserves_segment_breaks());
    }

    #[test]
    fn wrap_bits_match_the_spec_table() {
        // soft wrap: normal / pre-wrap / pre-line.
        assert!(WhiteSpace::Normal.wraps());
        assert!(WhiteSpace::PreWrap.wraps());
        assert!(WhiteSpace::PreLine.wraps());
        assert!(!WhiteSpace::Nowrap.wraps());
        assert!(!WhiteSpace::Pre.wraps());
    }
}
