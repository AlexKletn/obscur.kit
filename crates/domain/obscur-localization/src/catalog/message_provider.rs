#![doc = include_str!("message_provider.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::locale::LanguageTag;
use crate::message::{MessageKey, MessagePattern};

#[doc = include_str!("message_provider.doc/MessageProvider.md")]
pub trait MessageProvider {
    #[doc = include_str!("message_provider.doc/pattern.md")]
    #[must_use]
    fn pattern(&self, locale: &LanguageTag, key: &MessageKey) -> Option<&MessagePattern>;

    #[doc = include_str!("message_provider.doc/availability.md")]
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

#[doc = include_str!("message_provider.doc/EmptyMessageProvider.md")]
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
