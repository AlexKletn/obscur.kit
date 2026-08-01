#![doc = include_str!("grammatical_gender.doc/_module.md")]

use std::fmt;

#[doc = include_str!("grammatical_gender.doc/GrammaticalGender.md")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammaticalGender(Box<str>);

impl GrammaticalGender {
    #[doc = include_str!("grammatical_gender.doc/new.md")]
    #[must_use]
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    #[doc = include_str!("grammatical_gender.doc/as_str.md")]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[doc = include_str!("grammatical_gender.doc/masculine.md")]
    #[must_use]
    pub fn masculine() -> Self {
        Self::new("masculine")
    }

    #[doc = include_str!("grammatical_gender.doc/feminine.md")]
    #[must_use]
    pub fn feminine() -> Self {
        Self::new("feminine")
    }

    #[doc = include_str!("grammatical_gender.doc/neuter.md")]
    #[must_use]
    pub fn neuter() -> Self {
        Self::new("neuter")
    }

    #[doc = include_str!("grammatical_gender.doc/common.md")]
    #[must_use]
    pub fn common() -> Self {
        Self::new("common")
    }

    #[doc = include_str!("grammatical_gender.doc/animate.md")]
    #[must_use]
    pub fn animate() -> Self {
        Self::new("animate")
    }

    #[doc = include_str!("grammatical_gender.doc/inanimate.md")]
    #[must_use]
    pub fn inanimate() -> Self {
        Self::new("inanimate")
    }

    #[doc = include_str!("grammatical_gender.doc/personal.md")]
    #[must_use]
    pub fn personal() -> Self {
        Self::new("personal")
    }

    #[doc = include_str!("grammatical_gender.doc/cldr_values.md")]
    #[must_use]
    pub fn cldr_values() -> [Self; 7] {
        [
            Self::animate(),
            Self::common(),
            Self::feminine(),
            Self::inanimate(),
            Self::masculine(),
            Self::neuter(),
            Self::personal(),
        ]
    }
}

impl fmt::Display for GrammaticalGender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_spell_the_cldr_value_names() {
        assert_eq!(GrammaticalGender::masculine().as_str(), "masculine");
        assert_eq!(GrammaticalGender::feminine().as_str(), "feminine");
        assert_eq!(GrammaticalGender::neuter().as_str(), "neuter");
    }

    #[test]
    fn the_value_space_is_open() {
        // Polish's genders are not Russian's; a closed enum would have to pick.
        let polish = GrammaticalGender::new("personal");
        assert!(GrammaticalGender::cldr_values().contains(&polish));
        // ...and a value CLDR does not list is still constructible.
        assert_eq!(GrammaticalGender::new("epicene").as_str(), "epicene");
    }

    #[test]
    fn cldr_values_are_the_seven_from_the_vendored_file() {
        let values = GrammaticalGender::cldr_values();
        assert_eq!(values.len(), 7);
        let mut sorted = values.to_vec();
        sorted.sort();
        assert_eq!(sorted, values.to_vec(), "listed out of order");
    }
}
