#![doc = include_str!("plural_category.doc/_module.md")]

use std::fmt;

#[doc = include_str!("plural_category.doc/PluralCategory.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PluralCategory {
    #[doc = include_str!("plural_category.doc/Zero.md")]
    Zero,
    #[doc = include_str!("plural_category.doc/One.md")]
    One,
    #[doc = include_str!("plural_category.doc/Two.md")]
    Two,
    #[doc = include_str!("plural_category.doc/Few.md")]
    Few,
    #[doc = include_str!("plural_category.doc/Many.md")]
    Many,
    #[doc = include_str!("plural_category.doc/Other.md")]
    Other,
}

impl PluralCategory {
    #[doc = include_str!("plural_category.doc/ALL.md")]
    pub const ALL: [Self; 6] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Few,
        Self::Many,
        Self::Other,
    ];

    #[doc = include_str!("plural_category.doc/as_str.md")]
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

    #[doc = include_str!("plural_category.doc/from_keyword.md")]
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
