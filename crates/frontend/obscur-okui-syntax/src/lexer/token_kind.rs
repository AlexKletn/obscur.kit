#![doc = include_str!("token_kind.doc/_module.md")]

#[doc = include_str!("token_kind.doc/TokenKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    #[doc = include_str!("token_kind.doc/LAngle.md")]
    LAngle,
    #[doc = include_str!("token_kind.doc/RAngle.md")]
    RAngle,
    #[doc = include_str!("token_kind.doc/Slash.md")]
    Slash,
    #[doc = include_str!("token_kind.doc/Equals.md")]
    Equals,
    #[doc = include_str!("token_kind.doc/LBrace.md")]
    LBrace,
    #[doc = include_str!("token_kind.doc/RBrace.md")]
    RBrace,
    #[doc = include_str!("token_kind.doc/Colon.md")]
    Colon,
    #[doc = include_str!("token_kind.doc/Ident.md")]
    Ident,
    #[doc = include_str!("token_kind.doc/String.md")]
    String,
    #[doc = include_str!("token_kind.doc/Number.md")]
    Number,
    #[doc = include_str!("token_kind.doc/Text.md")]
    Text,
    #[doc = include_str!("token_kind.doc/Whitespace.md")]
    Whitespace,
    #[doc = include_str!("token_kind.doc/Error.md")]
    Error,
    #[doc = include_str!("token_kind.doc/Eof.md")]
    Eof,
}

impl TokenKind {
    #[doc = include_str!("token_kind.doc/is_trivia.md")]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    #[doc = include_str!("token_kind.doc/is_eof.md")]
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
