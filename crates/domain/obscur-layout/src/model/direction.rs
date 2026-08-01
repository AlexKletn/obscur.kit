#![doc = include_str!("direction.doc/_module.md")]

#[doc = include_str!("direction.doc/Direction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[doc = include_str!("direction.doc/Ltr.md")]
    #[default]
    Ltr,
    #[doc = include_str!("direction.doc/Rtl.md")]
    Rtl,
}

impl Direction {
    #[doc = include_str!("direction.doc/is_rtl.md")]
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
