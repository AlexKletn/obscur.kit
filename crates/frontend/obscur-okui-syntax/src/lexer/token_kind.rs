//! The lexer's alphabet: the kind of a single lexed token.

#[doc = include_str!("../../docs/api/lexer/token_kind.doc/TokenKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// `<` — opens a tag.
    LAngle,
    /// `>` — closes a tag.
    RAngle,
    /// `/` — self-closing/closing-tag marker.
    Slash,
    /// `=` — attribute assignment.
    Equals,
    /// `{` — opens an expression interpolation.
    LBrace,
    /// `}` — closes an expression interpolation.
    RBrace,
    /// `:` — namespace separator (e.g. `on:click`).
    Colon,
    /// An identifier or keyword: `[A-Za-z_][A-Za-z0-9_.-]*`.
    Ident,
    /// A double-quoted string literal, including the surrounding quotes.
    String,
    /// A numeric literal: `[0-9]+(\.[0-9]+)?`.
    Number,
    /// A run of raw text between markup constructs.
    Text,
    /// A run of insignificant whitespace (trivia).
    Whitespace,
    /// A single unrecognized character.
    Error,
    /// The synthetic end-of-input marker (empty text).
    Eof,
}

impl TokenKind {
    #[doc = include_str!("../../docs/api/lexer/token_kind.doc/is_trivia.md")]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    /// Returns `true` if this kind is the end-of-input marker.
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        matches!(self, Self::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_trivia_only_for_whitespace() {
        assert!(TokenKind::Whitespace.is_trivia());
        assert!(!TokenKind::Ident.is_trivia());
        assert!(!TokenKind::Eof.is_trivia());
    }

    #[test]
    fn is_eof_only_for_eof() {
        assert!(TokenKind::Eof.is_eof());
        assert!(!TokenKind::Whitespace.is_eof());
        assert!(!TokenKind::Text.is_eof());
    }
}
