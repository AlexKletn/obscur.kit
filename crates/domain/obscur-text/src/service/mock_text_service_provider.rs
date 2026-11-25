use std::collections::BTreeSet;

use crate::span::TextRange;

use super::{
    DiagnosticKind, DiagnosticSeverity, LanguageServiceRequest, ServiceKind, TextDiagnostic,
    TextServiceProvider,
};

#[doc = include_str!("../../docs/api/service/text_service_provider.doc/MockTextServiceProvider.md")]
#[derive(Debug, Clone)]
pub struct MockTextServiceProvider {
    /// The known-word dictionary, stored lowercased for case-insensitive lookup.
    known: BTreeSet<Box<str>>,
}

impl Default for MockTextServiceProvider {
    /// Same as [`MockTextServiceProvider::new`]: a provider seeded with the
    /// built-in word set, so the mock is usable out of the box.
    fn default() -> Self {
        Self::new()
    }
}

impl MockTextServiceProvider {
    /// Creates a provider seeded with a tiny built-in English word set.
    #[must_use]
    pub fn new() -> Self {
        let mut known = BTreeSet::new();
        for word in ["the", "quick", "brown", "fox", "hello", "world"] {
            known.insert(Box::from(word));
        }
        Self { known }
    }

    /// Adds `word` to the dictionary (lowercased), returning the updated provider.
    #[must_use]
    pub fn with_word(mut self, word: impl Into<Box<str>>) -> Self {
        let word = word.into();
        self.known
            .insert(word.to_ascii_lowercase().into_boxed_str());
        self
    }

    /// Returns whether `token` is in the dictionary, ignoring ASCII case.
    fn is_known(&self, token: &str) -> bool {
        self.known.contains(token.to_ascii_lowercase().as_str())
    }
}

#[doc = include_str!("../../docs/api/service/text_service_provider.doc/tokens.md")]
fn tokens(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (i, ch) in text.char_indices() {
        if ch.is_ascii_whitespace() {
            if let Some(s) = start.take() {
                out.push((s, &text[s..i]));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        out.push((s, &text[s..]));
    }
    out
}

impl TextServiceProvider for MockTextServiceProvider {
    fn supports(&self, kind: ServiceKind) -> bool {
        // Honest: the mock only spellchecks.
        matches!(kind, ServiceKind::Spellcheck)
    }

    fn analyze(&self, request: &LanguageServiceRequest) -> Vec<TextDiagnostic> {
        // The mock offers only spellcheck; honor the request's service selection.
        if !request.wants(ServiceKind::Spellcheck) {
            return Vec::new();
        }
        let base = request.range().start;
        let mut diagnostics = Vec::new();
        for (local_offset, token) in tokens(request.text()) {
            let doc_start = base + local_offset;
            // Skip tokens inside ignored / sensitive ranges (§5.13).
            if request.is_ignored(doc_start) {
                continue;
            }
            if self.is_known(token) {
                continue;
            }
            let doc_range = TextRange::new(doc_start, doc_start + token.len());
            diagnostics.push(TextDiagnostic::new(
                doc_range,
                DiagnosticKind::Spelling,
                DiagnosticSeverity::Warning,
                format!("unknown word: {token}"),
                request.version(),
            ));
        }
        diagnostics
    }

    fn detect_language(&self, text: &str) -> Option<Box<str>> {
        // Deterministic heuristic: if any known (English) word appears, call it
        // English; otherwise undetermined.
        if tokens(text).iter().any(|(_, tok)| self.is_known(tok)) {
            Some(Box::from("en"))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_supports_only_spellcheck() {
        let provider = MockTextServiceProvider::new();
        assert!(provider.supports(ServiceKind::Spellcheck));
        assert!(!provider.supports(ServiceKind::Grammar));
        assert!(!provider.supports(ServiceKind::Autocorrect));
        assert!(!provider.supports(ServiceKind::LanguageDetection));
    }

    #[test]
    fn mock_flags_unknown_word_with_document_byte_range() {
        let provider = MockTextServiceProvider::new();
        // The text slice starts at document offset 10, proving offset mapping.
        // "the zzz" -> "the" is known, "zzz" is unknown, local offset 4.
        let request = LanguageServiceRequest::new("the zzz", TextRange::new(10, 17), 42);
        let diagnostics = provider.analyze(&request);
        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(d.kind(), DiagnosticKind::Spelling);
        assert_eq!(d.severity(), DiagnosticSeverity::Warning);
        assert_eq!(d.message(), "unknown word: zzz");
        // Document range: base 10 + local 4 = 14, length 3 -> 14..17.
        assert_eq!(d.range(), TextRange::new(14, 17));
        // Version stamped from the request.
        assert_eq!(d.version(), 42);
    }

    #[test]
    fn mock_is_case_insensitive() {
        let provider = MockTextServiceProvider::new();
        let request = LanguageServiceRequest::new("The QUICK Fox", TextRange::new(0, 13), 0);
        // All three are known words regardless of case.
        assert!(provider.analyze(&request).is_empty());
    }

    #[test]
    fn mock_respects_ignored_ranges() {
        let provider = MockTextServiceProvider::new();
        // "zzz yyy": both unknown. Ignore the range covering the first token
        // (document offsets 0..3) so only the second is flagged.
        let request = LanguageServiceRequest::new("zzz yyy", TextRange::new(0, 7), 1)
            .ignoring(TextRange::new(0, 3));
        let diagnostics = provider.analyze(&request);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message(), "unknown word: yyy");
        assert_eq!(diagnostics[0].range(), TextRange::new(4, 7));
    }

    #[test]
    fn mock_respects_requested_services() {
        let provider = MockTextServiceProvider::new();
        // A request for only Grammar must yield no spelling diagnostics.
        let request = LanguageServiceRequest::new("zzz", TextRange::new(0, 3), 1)
            .requesting(ServiceKind::Grammar);
        assert!(provider.analyze(&request).is_empty());
    }

    #[test]
    fn mock_with_word_extends_dictionary() {
        let provider = MockTextServiceProvider::new().with_word("Rustacean");
        let request = LanguageServiceRequest::new("rustacean", TextRange::new(0, 9), 1);
        assert!(provider.analyze(&request).is_empty());
    }

    #[test]
    fn mock_detect_language() {
        let provider = MockTextServiceProvider::new();
        assert_eq!(
            provider.detect_language("hello there"),
            Some(Box::from("en"))
        );
        assert_eq!(provider.detect_language("zzz yyy"), None);
    }
}
