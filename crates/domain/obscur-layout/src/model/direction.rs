//! The CSS `direction` model: the inline base direction of a box's content.

#[doc = include_str!("../../docs/api/model/direction.doc/Direction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Left-to-right inline base direction (the CSS initial value).
    #[default]
    Ltr,
    /// Right-to-left inline base direction (`direction: rtl`).
    Rtl,
}

impl Direction {
    /// Whether this direction is [`Rtl`](Self::Rtl).
    #[must_use]
    pub const fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn default_is_ltr() {
        assert_eq!(Direction::default(), Direction::Ltr);
        assert!(!Direction::default().is_rtl());
    }

    #[test]
    fn rtl_reports_rtl() {
        assert!(Direction::Rtl.is_rtl());
    }
}
