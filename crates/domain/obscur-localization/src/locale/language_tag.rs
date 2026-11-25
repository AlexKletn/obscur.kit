//! `LanguageTag`: the current locale, and the fallback chain it resolves through
//! (spec §7.2).

use std::fmt;

use obscur_text::shaping::TextDirection;

use crate::locale::script_direction::{language_direction, script_direction};

#[doc = include_str!("../../docs/api/locale/language_tag.doc/LanguageTag.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageTag {
    /// The language subtag, canonical lower-case (2–3 alphabetic characters).
    language: Box<str>,
    /// The script subtag, canonical title-case (4 alphabetic characters).
    script: Option<Box<str>>,
    /// The region subtag, canonical upper-case (2 alphabetic) or 3 numeric.
    region: Option<Box<str>>,
}

impl LanguageTag {
    #[doc = include_str!("../../docs/api/locale/language_tag.doc/root.md")]
    #[must_use]
    pub fn root() -> Self {
        Self {
            language: "und".into(),
            script: None,
            region: None,
        }
    }

    #[doc = include_str!("../../docs/api/locale/language_tag.doc/parse.md")]
    #[must_use]
    pub fn parse(tag: &str) -> Option<Self> {
        let mut parts = tag.split('-');
        let language = parts.next()?;
        if !(2..=3).contains(&language.len()) || !language.chars().all(|c| c.is_ascii_alphabetic())
        {
            return None;
        }
        let mut script = None;
        let mut region = None;
        let mut next = parts.next();
        if let Some(part) = next
            && part.len() == 4
            && part.chars().all(|c| c.is_ascii_alphabetic())
        {
            script = Some(title_case(part));
            next = parts.next();
        }
        if let Some(part) = next
            && is_region(part)
        {
            region = Some(part.to_ascii_uppercase().into_boxed_str());
            next = parts.next();
        }
        // Anything left over is a subtag this type does not model. Refuse.
        if next.is_some() {
            return None;
        }
        Some(Self {
            language: language.to_ascii_lowercase().into_boxed_str(),
            script,
            region,
        })
    }

    /// Returns the canonical lower-case language subtag (`ru`, `en`, `und`).
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Returns the canonical title-case script subtag, if the tag names one.
    #[must_use]
    pub fn script(&self) -> Option<&str> {
        self.script.as_deref()
    }

    /// Returns the canonical upper-case region subtag, if the tag names one.
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Returns whether this is the undetermined [`root`](Self::root) locale.
    #[must_use]
    pub fn is_root(&self) -> bool {
        &*self.language == "und" && self.script.is_none() && self.region.is_none()
    }

    #[doc = include_str!("../../docs/api/locale/language_tag.doc/direction.md")]
    #[must_use]
    pub fn direction(&self) -> TextDirection {
        self.script
            .as_deref()
            .map_or_else(|| language_direction(&self.language), script_direction)
    }

    #[doc = include_str!("../../docs/api/locale/language_tag.doc/fallback_chain.md")]
    #[must_use]
    pub fn fallback_chain(&self) -> Vec<Self> {
        let mut chain = vec![self.clone()];
        if self.region.is_some() {
            chain.push(Self {
                language: self.language.clone(),
                script: self.script.clone(),
                region: None,
            });
        }
        if self.script.is_some() {
            chain.push(Self {
                language: self.language.clone(),
                script: None,
                region: None,
            });
        }
        let root = Self::root();
        if chain.last() != Some(&root) {
            chain.push(root);
        }
        chain
    }
}

impl Default for LanguageTag {
    /// The undetermined [`root`](Self::root) locale — *not* `en`.
    fn default() -> Self {
        Self::root()
    }
}

impl fmt::Display for LanguageTag {
    /// Writes the canonical `language[-Script][-REGION]` spelling, which
    /// [`parse`](Self::parse) round-trips.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.language)?;
        if let Some(script) = &self.script {
            write!(f, "-{script}")?;
        }
        if let Some(region) = &self.region {
            write!(f, "-{region}")?;
        }
        Ok(())
    }
}

/// Canonicalises a 4-letter script subtag to CLDR's title-case spelling.
fn title_case(script: &str) -> Box<str> {
    let mut out = String::with_capacity(script.len());
    for (i, c) in script.chars().enumerate() {
        if i == 0 {
            out.push(c.to_ascii_uppercase());
        } else {
            out.push(c.to_ascii_lowercase());
        }
    }
    out.into_boxed_str()
}

/// Whether `part` is a BCP-47 region subtag: two letters (`RU`) or three digits
/// (`419`).
fn is_region(part: &str) -> bool {
    (part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()))
        || (part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(s: &str) -> LanguageTag {
        LanguageTag::parse(s).expect("valid tag")
    }

    #[test]
    fn parses_language_only() {
        let t = tag("ru");
        assert_eq!(t.language(), "ru");
        assert_eq!(t.script(), None);
        assert_eq!(t.region(), None);
    }

    #[test]
    fn parses_and_canonicalises_every_subtag() {
        let t = tag("SR-latn-rs");
        assert_eq!(t.language(), "sr");
        assert_eq!(t.script(), Some("Latn"));
        assert_eq!(t.region(), Some("RS"));
        assert_eq!(t.to_string(), "sr-Latn-RS");
    }

    #[test]
    fn parses_numeric_region() {
        let t = tag("es-419");
        assert_eq!(t.region(), Some("419"));
        assert_eq!(t.to_string(), "es-419");
    }

    #[test]
    fn canonicalisation_makes_spellings_compare_equal() {
        assert_eq!(tag("RU-ru"), tag("ru-RU"));
    }

    #[test]
    fn rejects_what_it_does_not_model_rather_than_truncating() {
        for bad in [
            "",
            "e",
            "engl",
            "en-US-u-ca-buddhist",
            "de-DE-1996",
            "x-pig-latin",
            "ru_RU",
            "ru-R",
        ] {
            assert!(LanguageTag::parse(bad).is_none(), "{bad} should not parse");
        }
    }

    #[test]
    fn root_is_und_and_is_the_default() {
        assert!(LanguageTag::root().is_root());
        assert!(LanguageTag::default().is_root());
        assert_eq!(LanguageTag::default().to_string(), "und");
        // The Anglocentric default this whole module exists to refuse:
        assert_ne!(LanguageTag::default(), tag("en"));
    }

    #[test]
    fn fallback_chain_truncates_from_the_right_and_ends_at_root() {
        let chain: Vec<String> = tag("sr-Latn-RS")
            .fallback_chain()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(chain, ["sr-Latn-RS", "sr-Latn", "sr", "und"]);
    }

    #[test]
    fn fallback_chain_of_a_language_only_tag() {
        let chain: Vec<String> = tag("ru")
            .fallback_chain()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(chain, ["ru", "und"]);
    }

    #[test]
    fn fallback_chain_of_root_is_root_alone() {
        assert_eq!(LanguageTag::root().fallback_chain(), [LanguageTag::root()]);
    }

    #[test]
    fn fallback_chain_keeps_a_script_when_the_region_is_dropped() {
        let chain: Vec<String> = tag("az-Arab-IR")
            .fallback_chain()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(chain, ["az-Arab-IR", "az-Arab", "az", "und"]);
    }

    #[test]
    fn direction_follows_the_language_when_no_script_is_named() {
        assert_eq!(tag("ru").direction(), TextDirection::LeftToRight);
        assert_eq!(tag("en-US").direction(), TextDirection::LeftToRight);
        assert_eq!(tag("ar").direction(), TextDirection::RightToLeft);
        assert_eq!(tag("he-IL").direction(), TextDirection::RightToLeft);
        assert_eq!(tag("ur-PK").direction(), TextDirection::RightToLeft);
    }

    #[test]
    fn an_explicit_script_overrides_the_language_default() {
        // Both directions, and each half is chosen so the language table alone
        // gets it *wrong*: `az` is likely-Latn (LTR) and `ur` is likely-Arab
        // (RTL), so a `direction()` that ignored the script subtag would answer
        // LTR for the first and RTL for the second.
        assert_eq!(language_direction("az"), TextDirection::LeftToRight);
        assert_eq!(tag("az-Arab-IR").direction(), TextDirection::RightToLeft);

        assert_eq!(language_direction("ur"), TextDirection::RightToLeft);
        assert_eq!(tag("ur-Latn").direction(), TextDirection::LeftToRight);
    }

    #[test]
    fn root_is_left_to_right() {
        assert_eq!(LanguageTag::root().direction(), TextDirection::LeftToRight);
    }
}
