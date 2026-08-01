#![doc = include_str!("grammatical_case.doc/_module.md")]

use std::fmt;

#[doc = include_str!("grammatical_case.doc/GrammaticalCase.md")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammaticalCase(Box<str>);

impl GrammaticalCase {
    #[doc = include_str!("grammatical_case.doc/new.md")]
    #[must_use]
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    #[doc = include_str!("grammatical_case.doc/as_str.md")]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[doc = include_str!("grammatical_case.doc/nominative.md")]
    #[must_use]
    pub fn nominative() -> Self {
        Self::new("nominative")
    }

    #[doc = include_str!("grammatical_case.doc/genitive.md")]
    #[must_use]
    pub fn genitive() -> Self {
        Self::new("genitive")
    }

    #[doc = include_str!("grammatical_case.doc/dative.md")]
    #[must_use]
    pub fn dative() -> Self {
        Self::new("dative")
    }

    #[doc = include_str!("grammatical_case.doc/accusative.md")]
    #[must_use]
    pub fn accusative() -> Self {
        Self::new("accusative")
    }

    #[doc = include_str!("grammatical_case.doc/instrumental.md")]
    #[must_use]
    pub fn instrumental() -> Self {
        Self::new("instrumental")
    }

    #[doc = include_str!("grammatical_case.doc/prepositional.md")]
    #[must_use]
    pub fn prepositional() -> Self {
        Self::new("prepositional")
    }

    #[doc = include_str!("grammatical_case.doc/russian_values.md")]
    #[must_use]
    pub fn russian_values() -> [Self; 8] {
        [
            Self::nominative(),
            Self::genitive(),
            Self::dative(),
            Self::accusative(),
            Self::instrumental(),
            Self::prepositional(),
            Self::new("vocative"),
            Self::new("locative"),
        ]
    }
}

impl fmt::Display for GrammaticalCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_spell_the_cldr_value_names() {
        assert_eq!(GrammaticalCase::nominative().as_str(), "nominative");
        assert_eq!(GrammaticalCase::genitive().as_str(), "genitive");
        assert_eq!(GrammaticalCase::instrumental().as_str(), "instrumental");
    }

    #[test]
    fn russian_has_the_eight_cases_cldr_gives_it() {
        let values = GrammaticalCase::russian_values();
        assert_eq!(values.len(), 8);
        assert_eq!(values[0], GrammaticalCase::nominative());
        assert!(values.contains(&GrammaticalCase::new("vocative")));
        assert!(values.contains(&GrammaticalCase::prepositional()));
    }

    #[test]
    fn the_value_space_is_open_beyond_russian() {
        // Hungarian's, which Russian's list does not contain.
        let hungarian = GrammaticalCase::new("superessive");
        assert!(!GrammaticalCase::russian_values().contains(&hungarian));
        assert_eq!(hungarian.as_str(), "superessive");
    }
}
