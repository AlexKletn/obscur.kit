//! The replaceable port a [`Localizer`](crate::catalog::Localizer) reads messages
//! through (spec §7.2 replaceable provider).

use obscur_capability::SupportLevel;

use crate::locale::LanguageTag;
use crate::message::{MessageKey, MessagePattern};

/// The replaceable port for looking a message up (spec §7.2).
///
/// This is the hexagonal port §7.2 calls the *replaceable provider*: the domain
/// holds a `Box<dyn MessageProvider>` and never knows whether the patterns behind
/// it came from a compiled-in table, a `.ftl` file an adapter parsed, or a
/// translation service. Object-safe on purpose — no generics, no `Self`-returning
/// methods — so providers are stored polymorphically.
///
/// # Contract
///
/// - [`pattern`](Self::pattern) is asked for an **exact** locale, never a
///   language "close enough". Walking the fallback chain is the
///   [`Localizer`](crate::catalog::Localizer)'s job, and a provider that
///   second-guessed it would fall back twice.
/// - A provider reports an honest [`availability`](Self::availability) and never
///   panics.
///
/// The toolkit deliberately stops here. §7.2: *"The toolkit is not a full
/// application translation-management platform."* Extraction, merge, pluralisation
/// linting, translator workflow — all of that lives on the far side of this trait.
pub trait MessageProvider {
    /// Returns the pattern registered for `key` in exactly `locale`, or `None`.
    #[must_use]
    fn pattern(&self, locale: &LanguageTag, key: &MessageKey) -> Option<&MessagePattern>;

    /// Returns whether this provider can supply messages at all (spec §7.2).
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

/// A [`MessageProvider`] holding no messages.
///
/// The honest default for a [`Localizer`](crate::catalog::Localizer) nobody has
/// given a catalogue to: it reports [`SupportLevel::Unavailable`] and answers
/// `None` for everything, so a localizer over it renders every key as itself. That
/// is a documented reference implementation of the contract — visibly untranslated
/// beats invisibly English.
#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyMessageProvider;

impl MessageProvider for EmptyMessageProvider {
    fn pattern(&self, _locale: &LanguageTag, _key: &MessageKey) -> Option<&MessagePattern> {
        None
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_is_object_safe() {
        let _: Box<dyn MessageProvider> = Box::new(EmptyMessageProvider);
    }

    #[test]
    fn the_empty_provider_is_unavailable_and_answers_nothing() {
        let provider = EmptyMessageProvider;
        assert_eq!(provider.availability(), SupportLevel::Unavailable);
        assert!(
            provider
                .pattern(&LanguageTag::root(), &MessageKey::new("any"))
                .is_none()
        );
    }
}
