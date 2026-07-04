//! A syntactic error produced while parsing CSS (spec §4.13 diagnostics).

use crate::source::SourceSpan;

/// The category of a [`CssParseError`] (spec §4.13).
///
/// Each variant names a distinct way the CSS grammar can be violated, letting
/// callers group or filter diagnostics without matching on message text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssParseErrorKind {
    /// A token appeared where the grammar did not allow it.
    UnexpectedToken,
    /// Input ended while a construct was still open.
    UnexpectedEof,
    /// A selector could not be parsed.
    InvalidSelector,
    /// A declaration value could not be parsed.
    InvalidValue,
    /// A `:` was expected (for example between a property and its value).
    ExpectedColon,
    /// A `;` was expected to terminate a declaration.
    ExpectedSemicolon,
    /// A `{ ... }` block was left unterminated.
    UnterminatedBlock,
    /// A quoted string was left unterminated.
    UnterminatedString,
    /// An at-rule (for example `@media`) was malformed.
    InvalidAtRule,
}

impl CssParseErrorKind {
    /// Returns a short, stable label for this kind, used in [`CssParseError`]'s
    /// [`std::fmt::Display`] output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "unexpected token",
            Self::UnexpectedEof => "unexpected end of input",
            Self::InvalidSelector => "invalid selector",
            Self::InvalidValue => "invalid value",
            Self::ExpectedColon => "expected ':'",
            Self::ExpectedSemicolon => "expected ';'",
            Self::UnterminatedBlock => "unterminated block",
            Self::UnterminatedString => "unterminated string",
            Self::InvalidAtRule => "invalid at-rule",
        }
    }
}

/// A syntactic error produced while parsing CSS (spec §4.13).
///
/// Pairs a [`CssParseErrorKind`] with a human-readable `message` and the
/// [`SourceSpan`] the error covers. It implements [`std::error::Error`] so it
/// composes with the standard error ecosystem while remaining I/O-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssParseError {
    kind: CssParseErrorKind,
    message: Box<str>,
    span: SourceSpan,
}

impl CssParseError {
    /// Creates a parse error of `kind` with `message` covering `span`.
    #[must_use]
    pub fn new(kind: CssParseErrorKind, message: impl Into<Box<str>>, span: SourceSpan) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    /// Creates a [`CssParseErrorKind::UnexpectedToken`] error covering `span`.
    #[must_use]
    pub fn unexpected_token(message: impl Into<Box<str>>, span: SourceSpan) -> Self {
        Self::new(CssParseErrorKind::UnexpectedToken, message, span)
    }

    /// Creates a [`CssParseErrorKind::UnexpectedEof`] error at byte offset `at`
    /// (an empty span), with a default message.
    #[must_use]
    pub fn unexpected_eof(at: u32) -> Self {
        Self::new(
            CssParseErrorKind::UnexpectedEof,
            "unexpected end of input",
            SourceSpan::empty(at),
        )
    }

    /// Returns the category of this error.
    #[must_use]
    pub const fn kind(&self) -> CssParseErrorKind {
        self.kind
    }

    /// Returns the human-readable message describing this error.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the span this error covers.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl std::fmt::Display for CssParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} at {}..{}",
            self.kind.label(),
            self.message,
            self.span.start(),
            self.span.end()
        )
    }
}

impl std::error::Error for CssParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_kind_message_and_span() {
        let err = CssParseError::new(
            CssParseErrorKind::InvalidValue,
            "expected a color",
            SourceSpan::new(10, 15),
        );
        assert_eq!(err.to_string(), "invalid value: expected a color at 10..15");
    }

    #[test]
    fn usable_as_std_error() {
        let err = CssParseError::unexpected_token("stray '}'", SourceSpan::new(3, 4));
        let dyn_err: &dyn std::error::Error = &err;
        assert_eq!(dyn_err.to_string(), "unexpected token: stray '}' at 3..4");
        assert_eq!(err.kind(), CssParseErrorKind::UnexpectedToken);
    }

    #[test]
    fn unexpected_eof_uses_empty_span() {
        let err = CssParseError::unexpected_eof(42);
        assert_eq!(err.kind(), CssParseErrorKind::UnexpectedEof);
        assert_eq!(err.message(), "unexpected end of input");
        assert_eq!(err.span(), SourceSpan::empty(42));
        assert!(err.span().is_empty());
        assert_eq!(
            err.to_string(),
            "unexpected end of input: unexpected end of input at 42..42"
        );
    }
}
