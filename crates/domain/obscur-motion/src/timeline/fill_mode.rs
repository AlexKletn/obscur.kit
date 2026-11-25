//! [`FillMode`] — whether a timeline's value applies outside its active period
//! (spec §4.17; css-animations-1 §3.9 `animation-fill-mode`).

#[doc = include_str!("../../docs/api/timeline/fill_mode.doc/FillMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillMode {
    /// No fill: the value applies only while the timeline is running.
    #[default]
    None,
    /// Fills after the animation ends.
    Forwards,
    /// Fills during the delay, with the keyframe that starts the first
    /// iteration.
    Backwards,
    /// Both [`Forwards`](Self::Forwards) and [`Backwards`](Self::Backwards).
    Both,
}

impl FillMode {
    /// Returns whether this mode fills the delay period before the first
    /// iteration (css-animations-1 §3.9 `backwards`).
    #[must_use]
    pub const fn fills_backwards(self) -> bool {
        matches!(self, Self::Backwards | Self::Both)
    }

    /// Parses a CSS `<single-animation-fill-mode>` keyword, ASCII
    /// case-insensitively; [`None`](Option::None) for anything else.
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
