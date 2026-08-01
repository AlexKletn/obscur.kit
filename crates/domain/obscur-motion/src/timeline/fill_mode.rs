#![doc = include_str!("fill_mode.doc/_module.md")]

#[doc = include_str!("fill_mode.doc/FillMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillMode {
    #[doc = include_str!("fill_mode.doc/None.md")]
    #[default]
    None,
    #[doc = include_str!("fill_mode.doc/Forwards.md")]
    Forwards,
    #[doc = include_str!("fill_mode.doc/Backwards.md")]
    Backwards,
    #[doc = include_str!("fill_mode.doc/Both.md")]
    Both,
}

impl FillMode {
    #[doc = include_str!("fill_mode.doc/fills_backwards.md")]
    #[must_use]
    pub const fn fills_backwards(self) -> bool {
        matches!(self, Self::Backwards | Self::Both)
    }

    #[doc = include_str!("fill_mode.doc/from_keyword.md")]
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        if keyword.eq_ignore_ascii_case("none") {
            Some(Self::None)
        } else if keyword.eq_ignore_ascii_case("forwards") {
            Some(Self::Forwards)
        } else if keyword.eq_ignore_ascii_case("backwards") {
            Some(Self::Backwards)
        } else if keyword.eq_ignore_ascii_case("both") {
            Some(Self::Both)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_none() {
        assert_eq!(FillMode::default(), FillMode::None);
    }

    #[test]
    fn backwards_and_both_fill_the_delay() {
        assert!(FillMode::Backwards.fills_backwards());
        assert!(FillMode::Both.fills_backwards());
        assert!(!FillMode::None.fills_backwards());
        assert!(!FillMode::Forwards.fills_backwards());
    }

    #[test]
    fn keywords_parse_case_insensitively() {
        assert_eq!(FillMode::from_keyword("NONE"), Some(FillMode::None));
        assert_eq!(FillMode::from_keyword("Forwards"), Some(FillMode::Forwards));
        assert_eq!(
            FillMode::from_keyword("backwards"),
            Some(FillMode::Backwards)
        );
        assert_eq!(FillMode::from_keyword("both"), Some(FillMode::Both));
        assert_eq!(FillMode::from_keyword("running"), None);
    }
}
