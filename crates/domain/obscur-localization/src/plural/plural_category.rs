//! `PluralCategory`: the six CLDR plural categories (spec §7.2).

use std::fmt;

#[doc = include_str!("../../docs/api/plural/plural_category.doc/PluralCategory.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PluralCategory {
    /// `zero` — e.g. Arabic ٠.
    Zero,
    /// `one` — the singular *category*, which in Russian includes 21 and 101.
    One,
    /// `two` — the dual, e.g. Arabic.
    Two,
    /// `few` — the paucal, e.g. Russian 2–4, 22–24.
    Few,
    /// `many` — e.g. Russian 0, 5–20, 111.
    Many,
    /// `other` — the catch-all every language has; the only category of a
    /// language with no plural distinctions at all.
    Other,
}

impl PluralCategory {
    /// Every category, in CLDR's canonical order.
    pub const ALL: [Self; 6] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Few,
        Self::Many,
        Self::Other,
    ];

    #[doc = include_str!("../../docs/api/plural/plural_category.doc/as_str.md")]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    /// Parses a CLDR category keyword; `None` for anything else.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == keyword)
    }
}

impl fmt::Display for PluralCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip() {
        for category in PluralCategory::ALL {
            assert_eq!(
                PluralCategory::from_keyword(category.as_str()),
                Some(category)
            );
        }
    }

    #[test]
    fn unknown_keywords_are_rejected() {
        for bad in ["", "One", "plural", "singular", "several"] {
            assert!(PluralCategory::from_keyword(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn canonical_order() {
        assert!(PluralCategory::Zero < PluralCategory::One);
        assert!(PluralCategory::One < PluralCategory::Few);
        assert!(PluralCategory::Many < PluralCategory::Other);
    }
}
