#![doc = include_str!("text_direction.doc/_module.md")]

#[doc = include_str!("text_direction.doc/TextDirection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    #[doc = include_str!("text_direction.doc/LeftToRight.md")]
    #[default]
    LeftToRight,
    #[doc = include_str!("text_direction.doc/RightToLeft.md")]
    RightToLeft,
}

impl TextDirection {
    #[doc = include_str!("text_direction.doc/is_rtl.md")]
    #[must_use]
    pub const fn is_rtl(&self) -> bool {
        matches!(self, Self::RightToLeft)
    }

    #[doc = include_str!("text_direction.doc/is_ltr.md")]
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
