//! A typed, ranged, versioned language-service diagnostic (spec §5.13).

use crate::span::TextRange;

use super::{DiagnosticKind, DiagnosticSeverity};

#[doc = include_str!("../../docs/api/service/text_diagnostic.doc/TextDiagnostic.md")]
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
    #[doc = include_str!("../../docs/api/service/text_diagnostic.doc/new.md")]
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

    #[doc = include_str!("../../docs/api/service/text_diagnostic.doc/is_stale.md")]
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
}
