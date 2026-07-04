//! `ParseError`: a single diagnostic emitted by the [`Parser`](crate::parser::Parser),
//! plus the colocated [`ParseErrorKind`] classification.

use std::error::Error;
use std::fmt;

/// The classification of a [`ParseError`] (spec §2.9).
///
/// The parser is *error-tolerant*: it never hard-fails, so each recoverable
/// problem is recorded as one of these kinds and parsing continues over a
/// lossless tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    /// A token appeared where the grammar did not allow it.
    UnexpectedToken,
    /// Input ended while a construct was still open.
    UnexpectedEof,
    /// A `<` was not followed by a tag-name identifier.
    ExpectedTagName,
    /// An element's open tag was never closed (`>` / `/>` missing).
    ExpectedClose,
    /// A close tag's name did not match its open tag.
    MismatchedCloseTag,
    /// An attribute's `=` was not followed by a value.
    ExpectedAttributeValue,
    /// A `{` expression interpolation was never closed by a matching `}`.
    UnterminatedExpression,
}

impl ParseErrorKind {
    /// Returns a short, human-readable label for this kind.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::UnexpectedToken => "unexpected token",
            Self::UnexpectedEof => "unexpected end of input",
            Self::ExpectedTagName => "expected tag name",
            Self::ExpectedClose => "expected closing `>` or `/>`",
            Self::MismatchedCloseTag => "mismatched close tag",
            Self::ExpectedAttributeValue => "expected attribute value",
            Self::UnterminatedExpression => "unterminated expression",
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A single recoverable diagnostic produced while parsing `.okui` source
/// (spec §2.9).
///
/// A `ParseError` records *what* went wrong ([`ParseErrorKind`]), a
/// human-readable `message`, and the byte `offset` in the source where the
/// problem was detected. Errors never abort the parse: they are collected
/// alongside a lossless tree so tooling can report every problem at once.
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
