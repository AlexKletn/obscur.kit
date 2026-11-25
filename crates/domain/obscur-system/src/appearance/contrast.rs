//! Whether the desktop asks for increased contrast.

#[doc = include_str!("../../docs/api/appearance/contrast.doc/Contrast.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Contrast {
    /// No preference stated beyond the design's own contrast.
    Normal,
    /// More contrast than the default. Borders that were subtle become visible.
    High,
}

impl Contrast {
    /// Whether more contrast was asked for.
    #[must_use]
    pub const fn is_high(self) -> bool {
        matches!(self, Self::High)
    }

    #[doc = include_str!("../../docs/api/appearance/contrast.doc/css_keyword.md")]
    #[must_use]
    pub const fn css_keyword(self) -> &'static str {
        match self {
            Self::Normal => "no-preference",
            Self::High => "more",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_contrast_is_the_one_that_changes_a_design() {
        assert!(Contrast::High.is_high());
        assert!(!Contrast::Normal.is_high());
    }

    #[test]
    fn the_keywords_are_the_ones_prefers_contrast_uses() {
        // `@media (prefers-contrast: more)`, not our own vocabulary.
        assert_eq!(Contrast::High.css_keyword(), "more");
        assert_eq!(Contrast::Normal.css_keyword(), "no-preference");
    }
}
