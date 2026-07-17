//! `MessageCatalog`: the reference in-memory [`MessageProvider`] (spec §7.2).

use std::collections::HashMap;

use obscur_capability::SupportLevel;

use crate::catalog::MessageProvider;
use crate::locale::LanguageTag;
use crate::message::{MessageKey, MessagePattern, MessagePatternError};

/// An in-memory table of patterns, keyed by locale and message key.
///
/// The reference [`MessageProvider`]: a host builds one at start-up from whatever
/// its translations live in, and hands it to a
/// [`Localizer`](crate::catalog::Localizer). It reports
/// [`SupportLevel::Native`] once it holds anything and
/// [`SupportLevel::Unavailable`] while empty — an empty catalogue and no
/// catalogue are the same capability, and saying otherwise would be the kind of
/// cheerful lie this workspace's ports are built to avoid.
///
/// Patterns are parsed on insert, not on lookup: [`insert`](Self::insert) returns
/// the [`MessagePatternError`] so a broken translation is caught when the
/// catalogue is loaded rather than on the frame that first needs it.
#[derive(Debug, Default)]
pub struct MessageCatalog {
    locales: HashMap<LanguageTag, HashMap<MessageKey, MessagePattern>>,
}

impl MessageCatalog {
    /// Creates an empty catalogue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses `source` and registers it for `key` in `locale`, replacing any
    /// previous pattern.
    ///
    /// # Errors
    ///
    /// Returns the [`MessagePatternError`] when `source` is not a valid pattern;
    /// nothing is registered in that case.
    pub fn insert(
        &mut self,
        locale: &LanguageTag,
        key: impl Into<MessageKey>,
        source: &str,
    ) -> Result<(), MessagePatternError> {
        let pattern = MessagePattern::parse(source)?;
        self.locales
            .entry(locale.clone())
            .or_default()
            .insert(key.into(), pattern);
        Ok(())
    }

    /// Returns how many locales hold at least one message.
    #[must_use]
    pub fn locale_count(&self) -> usize {
        self.locales.len()
    }

    /// Returns whether the catalogue holds no messages at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.locales.values().all(HashMap::is_empty)
    }
}

impl MessageProvider for MessageCatalog {
    fn pattern(&self, locale: &LanguageTag, key: &MessageKey) -> Option<&MessagePattern> {
        // Exact locale only: the fallback chain belongs to the `Localizer`.
        self.locales.get(locale)?.get(key)
    }

    fn availability(&self) -> SupportLevel {
        if self.is_empty() {
            SupportLevel::Unavailable
        } else {
            SupportLevel::Native
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(s: &str) -> LanguageTag {
        LanguageTag::parse(s).expect("valid tag")
    }

    #[test]
    fn stores_and_returns_a_pattern_for_an_exact_locale() {
        let mut catalog = MessageCatalog::new();
        catalog
            .insert(&tag("ru"), "greet", "Привет, {name}!")
            .expect("valid");
        assert!(
            catalog
                .pattern(&tag("ru"), &MessageKey::new("greet"))
                .is_some()
        );
    }

    #[test]
    fn lookup_does_not_fall_back_on_its_own() {
        // `ru-RU` must miss even though `ru` hits: the chain is the localizer's,
        // and a provider that walked it too would fall back twice.
        let mut catalog = MessageCatalog::new();
        catalog
            .insert(&tag("ru"), "greet", "Привет")
            .expect("valid");
        assert!(
            catalog
                .pattern(&tag("ru-RU"), &MessageKey::new("greet"))
                .is_none()
        );
    }

    #[test]
    fn a_broken_pattern_is_refused_at_insert() {
        let mut catalog = MessageCatalog::new();
        let error = catalog.insert(&tag("ru"), "bad", "{n, plural, one{a}}");
        assert_eq!(error, Err(MessagePatternError::MissingOtherArm));
        assert!(catalog.is_empty(), "a refused insert must store nothing");
    }

    #[test]
    fn availability_tracks_whether_anything_is_in_it() {
        let mut catalog = MessageCatalog::new();
        assert_eq!(catalog.availability(), SupportLevel::Unavailable);
        catalog
            .insert(&tag("ru"), "greet", "Привет")
            .expect("valid");
        assert_eq!(catalog.availability(), SupportLevel::Native);
    }

    #[test]
    fn locales_are_independent() {
        let mut catalog = MessageCatalog::new();
        catalog.insert(&tag("ru"), "greet", "Привет").expect("ok");
        catalog.insert(&tag("en"), "greet", "Hello").expect("ok");
        assert_eq!(catalog.locale_count(), 2);
        assert!(
            catalog
                .pattern(&tag("de"), &MessageKey::new("greet"))
                .is_none()
        );
    }

    #[test]
    fn reinserting_a_key_replaces_it() {
        let mut catalog = MessageCatalog::new();
        catalog.insert(&tag("ru"), "greet", "Привет").expect("ok");
        catalog
            .insert(&tag("ru"), "greet", "Здравствуйте")
            .expect("ok");
        let pattern = catalog
            .pattern(&tag("ru"), &MessageKey::new("greet"))
            .expect("registered");
        assert_eq!(
            pattern,
            &MessagePattern::parse("Здравствуйте").expect("valid")
        );
    }
}
