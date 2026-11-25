use std::fmt;

#[doc = include_str!("../../docs/api/parser/parse_error.doc/ParseErrorKind.md")]
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
