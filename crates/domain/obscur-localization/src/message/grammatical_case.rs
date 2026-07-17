//! `GrammaticalCase`: a CLDR grammatical-case value (spec §7.2 case variants).

use std::fmt;

/// A grammatical case, as a CLDR value name.
///
/// Open for the same reason [`GrammaticalGender`](super::GrammaticalGender) is,
/// only more so: `specs/cldr/grammaticalFeatures.xml` lists **28** distinct case
/// values across its locales, and the eight Russian uses (`nominative genitive
/// dative accusative instrumental prepositional vocative locative`) are not a
/// superset of anyone else's — Hungarian alone contributes `delative`,
/// `sublative`, `superessive`, `terminative` and more.
///
/// Case variants matter to a UI toolkit for a plain reason: a message that
/// interpolates a noun cannot use the dictionary form everywhere. Russian «нет
/// {file}» needs the genitive of the word that «{file}» is nominative for. The
/// mechanism is the same as gender's — the case selects an arm by name — and the
/// *catalogue*, which is the only party that knows the actual word forms, holds
/// them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammaticalCase(Box<str>);

impl GrammaticalCase {
    /// Creates a case from a CLDR value name.
    #[must_use]
    pub fn new(value: impl Into<Box<str>>) -> Self {
        Self(value.into())
    }

    /// Returns the value name, as a message arm is spelled.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `nominative` — the dictionary form.
    #[must_use]
    pub fn nominative() -> Self {
        Self::new("nominative")
    }

    /// `genitive`.
    #[must_use]
    pub fn genitive() -> Self {
        Self::new("genitive")
    }

    /// `dative`.
    #[must_use]
    pub fn dative() -> Self {
        Self::new("dative")
    }

    /// `accusative`.
    #[must_use]
    pub fn accusative() -> Self {
        Self::new("accusative")
    }

    /// `instrumental`.
    #[must_use]
    pub fn instrumental() -> Self {
        Self::new("instrumental")
    }

    /// `prepositional`.
    #[must_use]
    pub fn prepositional() -> Self {
        Self::new("prepositional")
    }

    /// The eight cases `specs/cldr/grammaticalFeatures.xml` gives Russian, in the
    /// order that file lists them:
    ///
    /// ```text
    /// <grammaticalFeatures targets="nominal" locales="ru">
    ///   <grammaticalCase values="nominative genitive dative accusative instrumental prepositional vocative locative"/>
    ///   <grammaticalGender values="masculine feminine neuter"/>
    /// </grammaticalFeatures>
    /// ```
    ///
    /// Provided because §7.2 names Russian's case variants specifically. It is
    /// Russian's list, not *the* list — see the type docs.
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
