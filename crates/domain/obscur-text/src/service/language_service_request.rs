//! A request to analyze a text region with the language services (spec §5.13).

use crate::span::TextRange;

use super::ServiceKind;

#[doc = include_str!("../../docs/api/service/language_service_request.doc/LanguageServiceRequest.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServiceRequest {
    /// The text to analyze: the whole document or an incremental fragment.
    text: Box<str>,
    /// Where [`text`](Self::text) begins within the document, for mapping
    /// local offsets to document byte offsets.
    range: TextRange,
    /// The requested analyses; empty means "all" (see [`wants`](Self::wants)).
    services: Vec<ServiceKind>,
    /// Ignored / sensitive document ranges the provider must skip.
    ignored_ranges: Vec<TextRange>,
    /// The version of the text being analyzed.
    version: u64,
}

impl LanguageServiceRequest {
    #[doc = include_str!("../../docs/api/service/language_service_request.doc/new.md")]
    #[must_use]
    pub fn new(text: impl Into<Box<str>>, range: TextRange, version: u64) -> Self {
        Self {
            text: text.into(),
            range,
            services: Vec::new(),
            ignored_ranges: Vec::new(),
            version,
        }
    }

    #[doc = include_str!("../../docs/api/service/language_service_request.doc/with_services.md")]
    #[must_use]
    pub fn with_services(mut self, services: Vec<ServiceKind>) -> Self {
        self.services = services;
        self
    }

    /// Adds a single requested service, returning the updated request.
    #[must_use]
    pub fn requesting(mut self, kind: ServiceKind) -> Self {
        self.services.push(kind);
        self
    }

    /// Adds an ignored / sensitive range the provider must skip, returning the
    /// updated request.
    #[must_use]
    pub fn ignoring(mut self, range: TextRange) -> Self {
        self.ignored_ranges.push(range);
        self
    }

    /// Returns the text to analyze.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns where the text begins within the document.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the requested services; empty means "all".
    #[must_use]
    pub fn services(&self) -> &[ServiceKind] {
        &self.services
    }

    /// Returns the ignored / sensitive ranges the provider must skip.
    #[must_use]
    pub fn ignored_ranges(&self) -> &[TextRange] {
        &self.ignored_ranges
    }

    /// Returns the version of the text being analyzed.
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[doc = include_str!("../../docs/api/service/language_service_request.doc/wants.md")]
    #[must_use]
    pub fn wants(&self, kind: ServiceKind) -> bool {
        self.services.is_empty() || self.services.contains(&kind)
    }

    /// Returns whether the document byte `offset` falls within any ignored range
    /// (spec §5.13 ignored / sensitive ranges).
    #[must_use]
    pub fn is_ignored(&self, offset: usize) -> bool {
        self.ignored_ranges.iter().any(|r| r.contains(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_all_services_no_ignored() {
        let req = LanguageServiceRequest::new("hello world", TextRange::new(0, 11), 3);
        assert_eq!(req.text(), "hello world");
        assert_eq!(req.range(), TextRange::new(0, 11));
        assert_eq!(req.version(), 3);
        assert!(req.services().is_empty());
        assert!(req.ignored_ranges().is_empty());
    }

    #[test]
    fn wants_empty_means_all() {
        let req = LanguageServiceRequest::new("x", TextRange::empty(0), 0);
        assert!(req.wants(ServiceKind::Spellcheck));
        assert!(req.wants(ServiceKind::Grammar));
        assert!(req.wants(ServiceKind::Autocorrect));
        assert!(req.wants(ServiceKind::LanguageDetection));
    }

    #[test]
    fn wants_explicit_list() {
        let req = LanguageServiceRequest::new("x", TextRange::empty(0), 0)
            .requesting(ServiceKind::Grammar);
        assert!(req.wants(ServiceKind::Grammar));
        assert!(!req.wants(ServiceKind::Spellcheck));
    }

    #[test]
    fn with_services_replaces() {
        let req = LanguageServiceRequest::new("x", TextRange::empty(0), 0)
            .with_services(vec![ServiceKind::Spellcheck, ServiceKind::Autocorrect]);
        assert_eq!(
            req.services(),
            &[ServiceKind::Spellcheck, ServiceKind::Autocorrect]
        );
        assert!(req.wants(ServiceKind::Spellcheck));
        assert!(!req.wants(ServiceKind::Grammar));
    }

    #[test]
    fn is_ignored_checks_ranges() {
        let req = LanguageServiceRequest::new("x", TextRange::empty(0), 0)
            .ignoring(TextRange::new(4, 8))
            .ignoring(TextRange::new(20, 25));
        assert_eq!(req.ignored_ranges().len(), 2);
        assert!(!req.is_ignored(3));
        assert!(req.is_ignored(4));
        assert!(req.is_ignored(7));
        // Half-open: the end offset is excluded.
        assert!(!req.is_ignored(8));
        assert!(req.is_ignored(22));
    }
}
