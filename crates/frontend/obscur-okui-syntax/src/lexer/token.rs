//! A single lexed token: its kind, exact source slice and byte offset.

use crate::lexer::TokenKind;

#[doc = include_str!("../../docs/api/lexer/token.doc/Token.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    text: Box<str>,
    offset: u32,
}

impl Token {
    /// Creates a token of `kind` covering `text`, starting at byte `offset`.
    #[must_use]
    pub fn new(kind: TokenKind, text: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            text: text.into(),
            offset,
        }
    }

    /// Returns the token's kind.
    #[must_use]
    pub const fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Returns the exact source slice this token was lexed from.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the byte offset of the token's start in the source.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Returns the token's length in bytes.
    #[must_use]
    pub fn len(&self) -> u32 {
        u32::try_from(self.text.len()).unwrap_or(u32::MAX)
    }

    /// Returns `true` if the token covers no bytes (only the [`TokenKind::Eof`]
    /// marker is empty).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the byte offset just past the end of this token.
    #[must_use]
    pub fn end(&self) -> u32 {
        self.offset.saturating_add(self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_report_kind_text_and_offset() {
        let tok = Token::new(TokenKind::Ident, "Button", 3);
        assert_eq!(tok.kind(), TokenKind::Ident);
        assert_eq!(tok.text(), "Button");
        assert_eq!(tok.offset(), 3);
    }

    #[test]
    fn len_and_end_track_the_byte_range() {
        let tok = Token::new(TokenKind::Ident, "Button", 3);
        assert_eq!(tok.len(), 6);
        assert_eq!(tok.end(), 9);
        assert!(!tok.is_empty());
    }

    #[test]
    fn empty_eof_token_has_zero_len() {
        let eof = Token::new(TokenKind::Eof, "", 12);
        assert_eq!(eof.len(), 0);
        assert_eq!(eof.end(), 12);
        assert!(eof.is_empty());
    }
}
