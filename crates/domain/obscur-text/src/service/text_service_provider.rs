#![doc = include_str!("../../docs/api/service/text_service_provider.doc/_module.md")]

use super::{LanguageServiceRequest, ServiceKind, TextDiagnostic};

#[doc = include_str!("../../docs/api/service/text_service_provider.doc/TextServiceProvider.md")]
pub trait TextServiceProvider {
    /// Returns whether this provider offers the `kind` service (spec §5.13).
    #[must_use]
    fn supports(&self, kind: ServiceKind) -> bool;

    #[doc = include_str!("../../docs/api/service/text_service_provider.doc/analyze.md")]
    #[must_use]
    fn analyze(&self, request: &LanguageServiceRequest) -> Vec<TextDiagnostic>;

    /// Detects the language of `text`, returning a best-effort BCP-47 tag (e.g.
    /// `"en"`, `"pt-BR"`) or `None` when undetermined (spec §5.13).
    #[must_use]
    fn detect_language(&self, text: &str) -> Option<Box<str>>;
}

#[cfg(test)]
mod tests {
    use super::super::{MockTextServiceProvider, NullTextServiceProvider};
    use super::*;

    #[test]
    fn provider_is_object_safe() {
        let _: Box<dyn TextServiceProvider> = Box::new(MockTextServiceProvider::new());
        let _: Box<dyn TextServiceProvider> = Box::new(NullTextServiceProvider::new());
    }
}
