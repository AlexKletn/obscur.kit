//! `SyntaxToken`: a leaf of the lossless syntax tree.

use crate::lexer::Token;
use crate::syntax::SyntaxKind;

#[doc = include_str!("../../docs/api/syntax/syntax_token.doc/SyntaxToken.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    kind: SyntaxKind,
    text: Box<str>,
    offset: u32,
}

impl SyntaxToken {
    #[doc = include_str!("../../docs/api/syntax/syntax_token.doc/new.md")]
    #[must_use]
    pub fn new(kind: SyntaxKind, text: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            kind,
            text: text.into(),
            offset,
        }
    }

    /// Creates a leaf from a lexer [`Token`], mapping its kind via
    /// [`SyntaxKind::from_token`].
    #[must_use]
    pub fn from_lexer(token: &Token) -> Self {
        Self::new(
            SyntaxKind::from_token(token.kind()),
            token.text(),
            token.offset(),
        )
    }

    /// Returns the leaf's kind.
    #[must_use]
    pub const fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the exact source slice this leaf covers.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the byte offset of the leaf's start in the source.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Returns the leaf's length in bytes.
    #[must_use]
    pub fn len(&self) -> u32 {
        u32::try_from(self.text.len()).unwrap_or(u32::MAX)
    }

    /// Returns `true` if the leaf covers no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns `true` if this leaf is trivia (see [`SyntaxKind::is_trivia`]).
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        self.kind.is_trivia()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::TokenKind;

    #[test]
    fn from_lexer_copies_slice_offset_and_maps_kind() {
        let token = Token::new(TokenKind::Ident, "Button", 3);
        let leaf = SyntaxToken::from_lexer(&token);
        assert_eq!(leaf.kind(), SyntaxKind::Ident);
        assert_eq!(leaf.text(), "Button");
        assert_eq!(leaf.offset(), 3);
        assert_eq!(leaf.len(), 6);
        assert!(!leaf.is_trivia());
    }

    #[test]
    fn whitespace_leaf_is_trivia() {
        let leaf = SyntaxToken::new(SyntaxKind::Whitespace, " ", 0);
        assert!(leaf.is_trivia());
        assert!(!leaf.is_empty());
    }
}
