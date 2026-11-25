//! `ParseError`: a single diagnostic emitted by the [`Parser`](crate::parser::Parser),
//! plus the colocated [`ParseErrorKind`] classification.

use std::error::Error;
use std::fmt;

use super::ParseErrorKind;

#[doc = include_str!("../../docs/api/parser/parse_error.doc/ParseError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    message: Box<str>,
    offset: u32,
}

impl ParseError {
    /// Creates a diagnostic of `kind` with `message`, detected at byte
    /// `offset`.
    #[must_use]
    pub fn new(kind: ParseErrorKind, message: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            message: message.into(),
            offset,
        }
    }

    /// Returns the error's classification.
    #[must_use]
    pub const fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    /// Returns the human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the byte offset in the source where the problem was detected.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at byte {}: {}",
            self.kind.label(),
            self.offset,
            self.message
        )
    }
}

impl Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_includes_kind_offset_and_message() {
        let err = ParseError::new(ParseErrorKind::ExpectedTagName, "after `<`", 4);
        let text = err.to_string();
        assert!(text.contains("expected tag name"));
        assert!(text.contains('4'));
        assert!(text.contains("after `<`"));
    }

    #[test]
    fn accessors_report_fields() {
        let err = ParseError::new(ParseErrorKind::UnterminatedExpression, "missing }", 12);
        assert_eq!(err.kind(), ParseErrorKind::UnterminatedExpression);
        assert_eq!(err.message(), "missing }");
        assert_eq!(err.offset(), 12);
    }

    #[test]
    fn is_a_std_error() {
        let err = ParseError::new(ParseErrorKind::UnexpectedEof, "eof", 0);
        // Exercise the `Error` trait object path.
        let dyn_err: &dyn Error = &err;
        assert_eq!(dyn_err.to_string(), err.to_string());
    }

    #[test]
    fn errors_compare_by_value() {
        let a = ParseError::new(ParseErrorKind::UnexpectedToken, "x", 1);
        let b = ParseError::new(ParseErrorKind::UnexpectedToken, "x", 1);
        let c = ParseError::new(ParseErrorKind::UnexpectedToken, "x", 2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
