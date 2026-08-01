#![doc = include_str!("flex_wrap.doc/_module.md")]

#[doc = include_str!("flex_wrap.doc/FlexWrap.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlexWrap {
    #[doc = include_str!("flex_wrap.doc/NoWrap.md")]
    #[default]
    NoWrap,
    #[doc = include_str!("flex_wrap.doc/Wrap.md")]
    Wrap,
    #[doc = include_str!("flex_wrap.doc/WrapReverse.md")]
    WrapReverse,
}

impl FlexWrap {
    #[doc = include_str!("flex_wrap.doc/is_multi_line.md")]
    #[must_use]
    pub const fn is_multi_line(self) -> bool {
        matches!(self, Self::Wrap | Self::WrapReverse)
    }

    #[doc = include_str!("flex_wrap.doc/is_reverse.md")]
    #[must_use]
    pub const fn is_reverse(self) -> bool {
        matches!(self, Self::WrapReverse)
    }
}

#[cfg(test)]
mod tests {
    use super::FlexWrap;

    #[test]
    fn default_is_nowrap() {
        assert_eq!(FlexWrap::default(), FlexWrap::NoWrap);
    }

    #[test]
    fn multi_line_classification() {
        assert!(!FlexWrap::NoWrap.is_multi_line());
        assert!(FlexWrap::Wrap.is_multi_line());
        assert!(FlexWrap::WrapReverse.is_multi_line());
    }

    #[test]
    fn reverse_classification() {
        assert!(!FlexWrap::NoWrap.is_reverse());
        assert!(!FlexWrap::Wrap.is_reverse());
        assert!(FlexWrap::WrapReverse.is_reverse());
    }
}
