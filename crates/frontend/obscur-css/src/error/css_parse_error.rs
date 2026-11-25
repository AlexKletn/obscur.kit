//! A syntactic error produced while parsing CSS (spec §4.13 diagnostics).

use crate::source::SourceSpan;

use super::CssParseErrorKind;

#[doc = include_str!("../../docs/api/error/css_parse_error.doc/CssParseError.md")]
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
