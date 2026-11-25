//! The base (paragraph) writing direction of a text run (spec §5.10).

#[doc = include_str!("../../docs/api/shaping/text_direction.doc/TextDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    /// Left-to-right base direction (e.g. Latin, Cyrillic); the default.
    #[default]
    LeftToRight,
    /// Right-to-left base direction (e.g. Arabic, Hebrew).
    RightToLeft,
}

impl TextDirection {
    /// Returns whether this is the [`RightToLeft`](Self::RightToLeft) direction.
    #[must_use]
    pub const fn is_rtl(&self) -> bool {
        matches!(self, Self::RightToLeft)
    }

    /// Returns whether this is the [`LeftToRight`](Self::LeftToRight) direction.
    #[must_use]
    pub const fn is_ltr(&self) -> bool {
        matches!(self, Self::LeftToRight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_left_to_right() {
        assert_eq!(TextDirection::default(), TextDirection::LeftToRight);
    }

    #[test]
    fn direction_predicates() {
        assert!(TextDirection::LeftToRight.is_ltr());
        assert!(!TextDirection::LeftToRight.is_rtl());
        assert!(TextDirection::RightToLeft.is_rtl());
        assert!(!TextDirection::RightToLeft.is_ltr());
    }
}
