//! `GrammaticalGender`: a CLDR grammatical-gender value (spec §7.2 gender).

use std::fmt;

#[doc = include_str!("../../docs/api/message/grammatical_gender.doc/GrammaticalGender.md")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammaticalGender(Box<str>);

impl GrammaticalGender {
    /// Creates a gender from a CLDR value name.
    #[must_use]
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// Returns the value name, as a message arm is spelled.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `masculine` — CLDR value; used by e.g. `ru`, `de`, `es`.
    #[must_use]
    pub fn masculine() -> Self {
        Self::new("masculine")
    }

    /// `feminine` — CLDR value; used by e.g. `ru`, `pl`, `fr`.
    #[must_use]
    pub fn feminine() -> Self {
        Self::new("feminine")
    }

    /// `neuter` — CLDR value; used by e.g. `ru`, `pl`, `de`.
    #[must_use]
    pub fn neuter() -> Self {
        Self::new("neuter")
    }

    /// `common` — CLDR value; used by e.g. `da`, `sv`, `nl`.
    #[must_use]
    pub fn common() -> Self {
        Self::new("common")
    }

    /// `animate` — CLDR value; used by e.g. `pl`.
    #[must_use]
    pub fn animate() -> Self {
        Self::new("animate")
    }

    /// `inanimate` — CLDR value; used by e.g. `pl`.
    #[must_use]
    pub fn inanimate() -> Self {
        Self::new("inanimate")
    }

    /// `personal` — CLDR value; used by e.g. `pl`.
    #[must_use]
    pub fn personal() -> Self {
        Self::new("personal")
    }

    #[doc = include_str!("../../docs/api/message/grammatical_gender.doc/cldr_values.md")]
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
