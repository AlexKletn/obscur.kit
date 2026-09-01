#![doc = include_str!("color_scheme.doc/_module.md")]

#[doc = include_str!("color_scheme.doc/ColorScheme.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorScheme {
    #[doc = include_str!("color_scheme.doc/Light.md")]
    Light,
    #[doc = include_str!("color_scheme.doc/Dark.md")]
    Dark,
}

impl ColorScheme {
    #[doc = include_str!("color_scheme.doc/is_dark.md")]
    #[must_use]
    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    #[doc = include_str!("color_scheme.doc/css_keyword.md")]
    #[must_use]
    pub const fn css_keyword(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_schemes_are_told_apart() {
        assert!(ColorScheme::Dark.is_dark());
        assert!(!ColorScheme::Light.is_dark());
    }

    #[test]
    fn the_keywords_are_the_ones_css_uses() {
        // They go into `color-scheme` and into a custom property an author may
        // compare against, so they are the CSS spellings and not our own.
        assert_eq!(ColorScheme::Light.css_keyword(), "light");
        assert_eq!(ColorScheme::Dark.css_keyword(), "dark");
    }

    #[test]
    fn there_is_no_third_variant_for_no_preference() {
        // "The desktop did not say" is the absence of a scheme, not a scheme.
        // It is spelled `Option::None` at the field, so a caller that wants a
        // default has to write one down rather than inherit somebody's guess.
        let unstated: Option<ColorScheme> = None;
        assert!(unstated.is_none());
    }
}
