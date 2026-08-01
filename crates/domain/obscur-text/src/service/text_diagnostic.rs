#![doc = include_str!("text_diagnostic.doc/_module.md")]

use crate::span::TextRange;

#[doc = include_str!("text_diagnostic.doc/DiagnosticKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
    #[doc = include_str!("text_diagnostic.doc/Spelling.md")]
    Spelling,
    #[doc = include_str!("text_diagnostic.doc/Grammar.md")]
    Grammar,
    #[doc = include_str!("text_diagnostic.doc/Autocorrect.md")]
    Autocorrect,
    #[doc = include_str!("text_diagnostic.doc/Style.md")]
    Style,
}

#[doc = include_str!("text_diagnostic.doc/DiagnosticSeverity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    #[doc = include_str!("text_diagnostic.doc/Info.md")]
    Info,
    #[doc = include_str!("text_diagnostic.doc/Warning.md")]
    Warning,
    #[doc = include_str!("text_diagnostic.doc/Error.md")]
    Error,
}

#[doc = include_str!("text_diagnostic.doc/TextDiagnostic.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDiagnostic {
    #[doc = include_str!("text_diagnostic.doc/range.md")]
    range: TextRange,
    #[doc = include_str!("text_diagnostic.doc/kind.md")]
    kind: DiagnosticKind,
    #[doc = include_str!("text_diagnostic.doc/severity.md")]
    severity: DiagnosticSeverity,
    #[doc = include_str!("text_diagnostic.doc/message.md")]
    message: Box<str>,
    #[doc = include_str!("text_diagnostic.doc/suggestions.md")]
    suggestions: Vec<Box<str>>,
    #[doc = include_str!("text_diagnostic.doc/version.md")]
    version: u64,
}

impl TextDiagnostic {
    #[doc = include_str!("text_diagnostic.doc/new.md")]
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

    #[doc = include_str!("text_diagnostic.doc/with_suggestions.md")]
    #[must_use]
    pub fn with_suggestions(mut self, suggestions: Vec<Box<str>>) -> Self {
        self.suggestions = suggestions;
        self
    }

    #[doc = include_str!("text_diagnostic.doc/with_suggestion.md")]
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<Box<str>>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    #[doc = include_str!("text_diagnostic.doc/range_2.md")]
    #[must_use]
    pub const fn range(&self) -> TextRange {
        self.range
    }

    #[doc = include_str!("text_diagnostic.doc/kind_2.md")]
    #[must_use]
    pub const fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    #[doc = include_str!("text_diagnostic.doc/severity_2.md")]
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    #[doc = include_str!("text_diagnostic.doc/message_2.md")]
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[doc = include_str!("text_diagnostic.doc/suggestions_2.md")]
    #[must_use]
    pub fn suggestions(&self) -> &[Box<str>] {
        &self.suggestions
    }

    #[doc = include_str!("text_diagnostic.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[doc = include_str!("text_diagnostic.doc/has_suggestions.md")]
    #[must_use]
    pub fn has_suggestions(&self) -> bool {
        !self.suggestions.is_empty()
    }

    #[doc = include_str!("text_diagnostic.doc/is_stale.md")]
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
