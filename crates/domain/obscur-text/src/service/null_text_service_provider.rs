use super::{LanguageServiceRequest, ServiceKind, TextDiagnostic, TextServiceProvider};

#[doc = include_str!("../../docs/api/service/text_service_provider.doc/NullTextServiceProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct NullTextServiceProvider;

impl NullTextServiceProvider {
    /// Creates the no-op provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl TextServiceProvider for NullTextServiceProvider {
    fn supports(&self, _kind: ServiceKind) -> bool {
        false
    }

    fn analyze(&self, _request: &LanguageServiceRequest) -> Vec<TextDiagnostic> {
        Vec::new()
    }

    fn detect_language(&self, _text: &str) -> Option<Box<str>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::TextRange;

    #[test]
    fn null_provider_supplies_nothing() {
        let provider = NullTextServiceProvider::new();
        for kind in [
            ServiceKind::Spellcheck,
            ServiceKind::Grammar,
            ServiceKind::Autocorrect,
            ServiceKind::LanguageDetection,
        ] {
            assert!(!provider.supports(kind));
        }
        let request = LanguageServiceRequest::new("anything here", TextRange::new(0, 13), 1);
        assert!(provider.analyze(&request).is_empty());
        assert_eq!(provider.detect_language("hello world"), None);
    }
}
