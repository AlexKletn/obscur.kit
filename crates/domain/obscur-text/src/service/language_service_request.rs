//! A request to analyze a text region with the language services (spec §5.13).

use crate::span::TextRange;

/// Which analysis a [`LanguageServiceRequest`] asks a provider to run
/// (spec §5.13 optional language services).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceKind {
    /// Spellcheck: flag misspelled words.
    Spellcheck,
    /// Grammar: flag grammatical issues.
    Grammar,
    /// Autocorrect: propose replacements for likely typos.
    Autocorrect,
    /// Language detection: identify the text's language.
    LanguageDetection,
}

/// A request to analyze a region of a document with the language services
/// (spec §5.13 optional language services).
///
/// Analysis is **incremental and versioned** (§5.13): [`text`](Self::text) is the
/// slice to analyze — the whole document or an incremental fragment —
/// [`range`](Self::range) locates that slice within the document (so diagnostics
/// can be mapped back to document byte offsets), and [`version`](Self::version)
/// records the text version so the resulting diagnostics can be dated.
///
/// [`services`](Self::services) selects which analyses to run — an **empty** list
/// means "all" (see [`wants`](Self::wants)). [`ignored_ranges`](Self::ignored_ranges)
/// lists **ignored / sensitive** document regions a provider must skip (§5.13),
/// for example code spans, URLs, or password fields; an offset inside any ignored
/// range is reported by [`is_ignored`](Self::is_ignored).
///
/// This type is pure data: it carries a request but performs no analysis; a
/// [`TextServiceProvider`] consumes it.
///
/// [`TextServiceProvider`]: super::TextServiceProvider
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
    /// Creates a request to analyze `text`, located at `range` in the document
    /// and stamped with `version` (spec §5.13).
    ///
    /// The request initially requests **all** services (an empty
    /// [`services`](Self::services) list) and has no
    /// [`ignored_ranges`](Self::ignored_ranges); narrow it with
    /// [`with_services`](Self::with_services) / [`requesting`](Self::requesting)
    /// and [`ignoring`](Self::ignoring).
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

    /// Replaces the requested services, returning the updated request.
    ///
    /// An empty list means "all services" (see [`wants`](Self::wants)).
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

    /// Returns whether `kind` is requested (spec §5.13).
    ///
    /// An empty [`services`](Self::services) list requests every service, so this
    /// returns `true` for any `kind` in that case; otherwise it returns whether
    /// `kind` is explicitly listed.
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
