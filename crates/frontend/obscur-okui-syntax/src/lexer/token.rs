#![doc = include_str!("token.doc/_module.md")]

use crate::lexer::TokenKind;

#[doc = include_str!("token.doc/Token.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    kind: TokenKind,
    text: Box<str>,
    offset: u32,
}

impl Token {
    #[doc = include_str!("token.doc/new.md")]
    #[must_use]
    pub fn new(kind: TokenKind, text: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            text: text.into(),
            offset,
        }
    }

    #[doc = include_str!("token.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> TokenKind {
        self.kind
    }

    #[doc = include_str!("token.doc/text.md")]
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[doc = include_str!("token.doc/offset.md")]
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    #[doc = include_str!("token.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> u32 {
        u32::try_from(self.text.len()).unwrap_or(u32::MAX)
    }

    #[doc = include_str!("token.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[doc = include_str!("token.doc/end.md")]
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
