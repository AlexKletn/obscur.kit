#![doc = include_str!("css_parse_error.doc/_module.md")]

use crate::source::SourceSpan;

#[doc = include_str!("css_parse_error.doc/CssParseErrorKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssParseErrorKind {
    #[doc = include_str!("css_parse_error.doc/UnexpectedToken.md")]
    UnexpectedToken,
    #[doc = include_str!("css_parse_error.doc/UnexpectedEof.md")]
    UnexpectedEof,
    #[doc = include_str!("css_parse_error.doc/InvalidSelector.md")]
    InvalidSelector,
    #[doc = include_str!("css_parse_error.doc/InvalidValue.md")]
    InvalidValue,
    #[doc = include_str!("css_parse_error.doc/ExpectedColon.md")]
    ExpectedColon,
    #[doc = include_str!("css_parse_error.doc/ExpectedSemicolon.md")]
    ExpectedSemicolon,
    #[doc = include_str!("css_parse_error.doc/UnterminatedBlock.md")]
    UnterminatedBlock,
    #[doc = include_str!("css_parse_error.doc/UnterminatedString.md")]
    UnterminatedString,
    #[doc = include_str!("css_parse_error.doc/InvalidAtRule.md")]
    InvalidAtRule,
}

impl CssParseErrorKind {
    #[doc = include_str!("css_parse_error.doc/label.md")]
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

#[doc = include_str!("css_parse_error.doc/CssParseError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssParseError {
    kind: CssParseErrorKind,
    message: Box<str>,
    span: SourceSpan,
}

impl CssParseError {
    #[doc = include_str!("css_parse_error.doc/new.md")]
    #[must_use]
    pub fn new(kind: CssParseErrorKind, message: impl Into<Box<str>>, span: SourceSpan) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    #[doc = include_str!("css_parse_error.doc/unexpected_token.md")]
    #[must_use]
    pub fn unexpected_token(message: impl Into<Box<str>>, span: SourceSpan) -> Self {
        Self::new(CssParseErrorKind::UnexpectedToken, message, span)
    }

    #[doc = include_str!("css_parse_error.doc/unexpected_eof.md")]
    #[must_use]
    pub fn unexpected_eof(at: u32) -> Self {
        Self::new(
            CssParseErrorKind::UnexpectedEof,
            "unexpected end of input",
            SourceSpan::empty(at),
        )
    }

    #[doc = include_str!("css_parse_error.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> CssParseErrorKind {
        self.kind
    }

    #[doc = include_str!("css_parse_error.doc/message.md")]
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[doc = include_str!("css_parse_error.doc/span.md")]
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
