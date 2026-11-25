//! The replaceable port a [`Localizer`](crate::catalog::Localizer) reads messages
//! through (spec §7.2 replaceable provider).

use obscur_capability::SupportLevel;

use crate::locale::LanguageTag;
use crate::message::{MessageKey, MessagePattern};

#[doc = include_str!("../../docs/api/catalog/message_provider.doc/MessageProvider.md")]
pub trait MessageProvider {
    /// Returns the pattern registered for `key` in exactly `locale`, or `None`.
    #[must_use]
    fn pattern(&self, locale: &LanguageTag, key: &MessageKey) -> Option<&MessagePattern>;

    /// Returns whether this provider can supply messages at all (spec §7.2).
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

#[cfg(test)]
mod tests {
    use super::super::EmptyMessageProvider;
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
