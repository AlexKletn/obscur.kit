#[doc = include_str!("../../docs/api/error/css_parse_error.doc/CssParseErrorKind.md")]
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
