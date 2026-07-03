//! The replaceable port for optional language services (spec §5.13).
//!
//! This module defines [`TextServiceProvider`], the hexagonal port through which
//! the text domain requests optional language analyses — spellcheck, grammar,
//! autocorrect, language detection, and user-dictionary lookups (spec §5.13).
//! Interface adapters wrap a real linguistic engine behind it; the domain never
//! depends on one, so the honest default ([`NullTextServiceProvider`]) supplies
//! nothing and no engine ships in the core package.
//!
//! Two reference implementations are colocated: [`NullTextServiceProvider`], the
//! no-op honest default, and [`MockTextServiceProvider`], a deterministic
//! in-memory spellchecker used to exercise the port in tests. Neither is a real
//! linguistic engine.

use std::collections::BTreeSet;

use crate::span::TextRange;

use super::language_service_request::{LanguageServiceRequest, ServiceKind};
use super::text_diagnostic::{DiagnosticKind, DiagnosticSeverity, TextDiagnostic};

/// The replaceable port for optional language services (spec §5.13).
///
/// A provider may supply any subset of the §5.13 services — spellcheck, grammar,
/// autocorrect, language detection, and user-dictionary lookups — and reports
/// what it offers through [`supports`](Self::supports). Multiple providers may be
/// installed; the engine coordinating them is the caller's concern.
///
/// The trait is deliberately **object-safe** — no generic or `Self`-returning
/// methods — so adapters can be held as `Box<dyn TextServiceProvider>`.
///
/// The domain contract is a **pure function** from a
/// [`LanguageServiceRequest`] to a [`Vec<TextDiagnostic>`]: given a request,
/// [`analyze`](Self::analyze) returns the diagnostics for it. The §5.13
/// *asynchronous* and *incremental* delivery of those diagnostics — running the
/// engine off-thread, streaming partial results, coalescing versions — is the
/// adapter's responsibility and lives outside this pure-domain port.
pub trait TextServiceProvider {
    /// Returns whether this provider offers the `kind` service (spec §5.13).
    #[must_use]
    fn supports(&self, kind: ServiceKind) -> bool;

    /// Analyzes `request` and returns its diagnostics (spec §5.13).
    ///
    /// The returned [`TextDiagnostic`]s are typed, ranged (in **document** byte
    /// offsets), and stamped with [`request.version()`](LanguageServiceRequest::version).
    /// An implementation MUST:
    ///
    /// - only run a service the request asks for
    ///   ([`request.wants`](LanguageServiceRequest::wants)); and
    /// - skip any issue whose position falls in an ignored / sensitive range
    ///   ([`request.is_ignored`](LanguageServiceRequest::is_ignored)).
    ///
    /// Asynchronous and incremental delivery is an adapter concern; here the
    /// contract is the pure mapping request → diagnostics.
    #[must_use]
    fn analyze(&self, request: &LanguageServiceRequest) -> Vec<TextDiagnostic>;

    /// Detects the language of `text`, returning a best-effort BCP-47 tag (e.g.
    /// `"en"`, `"pt-BR"`) or `None` when undetermined (spec §5.13).
    #[must_use]
    fn detect_language(&self, text: &str) -> Option<Box<str>>;
}

/// A reference, no-op [`TextServiceProvider`] that supplies nothing — the honest
/// default (spec §5.13).
///
/// Per §5.13 no linguistic engine is required in the core package, so the default
/// provider offers no service: [`supports`](TextServiceProvider::supports) is
/// `false` for every kind, [`analyze`](TextServiceProvider::analyze) returns an
/// empty [`Vec`], and [`detect_language`](TextServiceProvider::detect_language)
/// returns `None`. Its "nothing" answers are correct behavior, not stubs waiting
/// to be filled in; a real adapter replaces it.
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

/// A deterministic, in-memory spellchecking [`TextServiceProvider`] for tests and
/// reference use (spec §5.13).
///
/// This is a **test / reference stub, not a real linguistic engine**. It supports
/// only [`ServiceKind::Spellcheck`], checking each whitespace-delimited token of
/// the request text against an in-memory [`known`](Self::known) dictionary
/// (case-insensitive). Any token absent from the dictionary is flagged with a
/// [`DiagnosticKind::Spelling`] / [`DiagnosticSeverity::Warning`] diagnostic whose
/// range is the token's **document** byte range (its offset within the request
/// text, translated by [`request.range().start`](LanguageServiceRequest::range)).
///
/// It is fully deterministic: the dictionary is a [`BTreeSet`] and tokens are
/// scanned left to right, so a given request always yields the same ordered
/// diagnostics. [`detect_language`](TextServiceProvider::detect_language) returns
/// `Some("en")` when any dictionary word appears, else `None`.
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
        self.known.insert(word.to_ascii_lowercase().into_boxed_str());
        self
    }

    /// Returns whether `token` is in the dictionary, ignoring ASCII case.
    fn is_known(&self, token: &str) -> bool {
        self.known
            .contains(token.to_ascii_lowercase().as_str())
    }
}

/// Splits `text` into whitespace-delimited tokens, pairing each with its byte
/// offset within `text`.
///
/// Tokens are runs of non-[`char::is_ascii_whitespace`] characters; the offset is
/// the byte position where the run starts. The scan is left-to-right, so the
/// order is deterministic.
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
    fn provider_is_object_safe() {
        let _: Box<dyn TextServiceProvider> = Box::new(MockTextServiceProvider::new());
        let _: Box<dyn TextServiceProvider> = Box::new(NullTextServiceProvider::new());
    }

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
