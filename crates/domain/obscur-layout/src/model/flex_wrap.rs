//! Whether a flex container is single-line or multi-line (`flex-wrap`).

#[doc = include_str!("../../docs/api/model/flex_wrap.doc/FlexWrap.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlexWrap {
    /// Single-line: all items share one main-axis line (the default).
    #[default]
    NoWrap,
    /// Multi-line: items wrap onto successive lines stacked along the cross axis.
    Wrap,
    /// Multi-line, with the lines stacked in reverse cross-axis order.
    WrapReverse,
}

impl FlexWrap {
    /// Returns `true` when the container wraps onto multiple lines (either
    /// [`Wrap`](FlexWrap::Wrap) or [`WrapReverse`](FlexWrap::WrapReverse)).
    #[must_use]
    pub const fn is_multi_line(self) -> bool {
        matches!(self, Self::Wrap | Self::WrapReverse)
    }

    /// Returns `true` when wrapped lines are stacked in reverse cross-axis order
    /// ([`WrapReverse`](FlexWrap::WrapReverse)).
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
