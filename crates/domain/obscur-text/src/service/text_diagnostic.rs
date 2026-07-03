//! A typed, ranged, versioned language-service diagnostic (spec §5.13).

use crate::span::TextRange;

/// The category of a [`TextDiagnostic`] — which language service produced it
/// (spec §5.13 optional language services).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
    /// A misspelled word flagged by spellcheck.
    Spelling,
    /// A grammatical issue flagged by the grammar checker.
    Grammar,
    /// An autocorrect suggestion (a likely typo with a replacement).
    Autocorrect,
    /// A stylistic remark (wordiness, tone, etc.).
    Style,
}

/// The severity of a [`TextDiagnostic`], ordered least-to-most severe
/// (spec §5.13).
///
/// The [`PartialOrd`]/[`Ord`] derivation follows declaration order, so
/// [`Info`](Self::Info) `<` [`Warning`](Self::Warning) `<` [`Error`](Self::Error);
/// a consumer can filter by a minimum severity threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    /// An informational note; no action implied.
    Info,
    /// A warning the user may wish to address.
    Warning,
    /// An error the user is likely expected to fix.
    Error,
}

/// A typed, ranged, versioned diagnostic emitted by a language service
/// (spec §5.13 optional language services).
///
/// Diagnostics are **typed** ([`kind`](Self::kind)/[`severity`](Self::severity)),
/// **ranged** ([`range`](Self::range) names the flagged bytes of the document),
/// and **versioned** ([`version`](Self::version) records the document/text
/// version the diagnostic was computed against). Because analysis is asynchronous
/// and incremental (§5.13), a diagnostic can arrive after the document has moved
/// on; a consumer discards a diagnostic whose version is stale relative to the
/// current document version (see [`is_stale`](Self::is_stale)).
///
/// Each diagnostic may carry replacement [`suggestions`](Self::suggestions)
/// (e.g. spelling candidates), most-preferred first. This type is pure data: it
/// performs no analysis and holds no engine state — a [`TextServiceProvider`]
/// produces it.
///
/// [`TextServiceProvider`]: super::TextServiceProvider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDiagnostic {
    /// The byte range in the document that this diagnostic flags.
    range: TextRange,
    /// The category of the diagnostic.
    kind: DiagnosticKind,
    /// The severity of the diagnostic.
    severity: DiagnosticSeverity,
    /// A human-readable description of the issue.
    message: Box<str>,
    /// Replacement candidates, most-preferred first; may be empty.
    suggestions: Vec<Box<str>>,
    /// The document/text version this diagnostic was computed against.
    version: u64,
}

impl TextDiagnostic {
    /// Creates a diagnostic flagging `range`, stamped with `version` (spec §5.13).
    ///
    /// The diagnostic starts with no [`suggestions`](Self::suggestions); attach
    /// them with [`with_suggestions`](Self::with_suggestions) or
    /// [`with_suggestion`](Self::with_suggestion).
    #[must_use]
    pub fn new(
        range: TextRange,
        kind: DiagnosticKind,
        severity: DiagnosticSeverity,
        message: impl Into<Box<str>>,
        version: u64,
    ) -> Self {
        Self {
            range,
            kind,
            severity,
            message: message.into(),
            suggestions: Vec::new(),
            version,
        }
    }

    /// Replaces the replacement suggestions, returning the updated diagnostic.
    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<Box<str>>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// Appends a single replacement suggestion, returning the updated diagnostic.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<Box<str>>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// Returns the byte range in the document this diagnostic flags.
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    /// Returns the category of the diagnostic.
    #[must_use]
    pub const fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    /// Returns the severity of the diagnostic.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the human-readable description of the issue.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the replacement candidates, most-preferred first.
    #[must_use]
    pub fn suggestions(&self) -> &[Box<str>] {
        &self.suggestions
    }

    /// Returns the document/text version this diagnostic was computed against.
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Returns whether this diagnostic carries any replacement suggestions.
    #[must_use]
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }

    /// Returns whether this diagnostic is stale relative to `current_version`
    /// (spec §5.13 versioned diagnostics).
    ///
    /// A diagnostic is stale when it was computed against an older document
    /// version (`self.version < current_version`); a consumer discards stale
    /// diagnostics because the flagged bytes may have since moved or changed.
    #[must_use]
    pub const fn is_stale(&self, current_version: u64) -> bool {
        self.version < current_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_fields_and_has_no_suggestions() {
        let d = TextDiagnostic::new(
            TextRange::new(4, 9),
            DiagnosticKind::Spelling,
            DiagnosticSeverity::Warning,
            "unknown word: hte",
            7,
        );
        assert_eq!(d.range(), TextRange::new(4, 9));
        assert_eq!(d.kind(), DiagnosticKind::Spelling);
        assert_eq!(d.severity(), DiagnosticSeverity::Warning);
        assert_eq!(d.message(), "unknown word: hte");
        assert_eq!(d.version(), 7);
        assert!(!d.has_suggestions());
        assert!(d.suggestions().is_empty());
    }

    #[test]
    fn with_suggestions_and_with_suggestion() {
        let d = TextDiagnostic::new(
            TextRange::new(0, 3),
            DiagnosticKind::Autocorrect,
            DiagnosticSeverity::Info,
            "the",
            1,
        )
        .with_suggestions(vec![Box::from("the"), Box::from("then")])
        .with_suggestion("they");
        assert!(d.has_suggestions());
        assert_eq!(
            d.suggestions(),
            &[Box::from("the"), Box::from("then"), Box::from("they")]
        );
    }

    #[test]
    fn is_stale_compares_versions() {
        let d = TextDiagnostic::new(
            TextRange::empty(0),
            DiagnosticKind::Grammar,
            DiagnosticSeverity::Error,
            "msg",
            5,
        );
        // Older than current -> stale.
        assert!(d.is_stale(6));
        // Equal -> current, not stale.
        assert!(!d.is_stale(5));
        // Newer than current (shouldn't happen) -> not stale.
        assert!(!d.is_stale(4));
    }

    #[test]
    fn severity_orders_least_to_most_severe() {
        assert!(DiagnosticSeverity::Info < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Error);
    }
}
