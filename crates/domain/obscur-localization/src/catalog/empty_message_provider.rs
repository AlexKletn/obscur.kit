use obscur_capability::SupportLevel;

use crate::locale::LanguageTag;
use crate::message::{MessageKey, MessagePattern};

use super::MessageProvider;

#[doc = include_str!("../../docs/api/catalog/message_provider.doc/EmptyMessageProvider.md")]
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
