#![doc = include_str!("parse_error.doc/_module.md")]

use std::error::Error;
use std::fmt;

#[doc = include_str!("parse_error.doc/ParseErrorKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    #[doc = include_str!("parse_error.doc/UnexpectedToken.md")]
    UnexpectedToken,
    #[doc = include_str!("parse_error.doc/UnexpectedEof.md")]
    UnexpectedEof,
    #[doc = include_str!("parse_error.doc/ExpectedTagName.md")]
    ExpectedTagName,
    #[doc = include_str!("parse_error.doc/ExpectedClose.md")]
    ExpectedClose,
    #[doc = include_str!("parse_error.doc/MismatchedCloseTag.md")]
    MismatchedCloseTag,
    #[doc = include_str!("parse_error.doc/ExpectedAttributeValue.md")]
    ExpectedAttributeValue,
    #[doc = include_str!("parse_error.doc/UnterminatedExpression.md")]
    UnterminatedExpression,
}

impl ParseErrorKind {
    #[doc = include_str!("parse_error.doc/label.md")]
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

#[doc = include_str!("parse_error.doc/ParseError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    message: Box<str>,
    offset: u32,
}

impl ParseError {
    #[doc = include_str!("parse_error.doc/new.md")]
    #[must_use]
    pub fn new(kind: ParseErrorKind, message: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            message: message.into(),
            offset,
        }
    }

    #[doc = include_str!("parse_error.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    #[doc = include_str!("parse_error.doc/message.md")]
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[doc = include_str!("parse_error.doc/offset.md")]
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
