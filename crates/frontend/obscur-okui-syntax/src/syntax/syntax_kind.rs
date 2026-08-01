#![doc = include_str!("syntax_kind.doc/_module.md")]

use crate::lexer::TokenKind;

#[doc = include_str!("syntax_kind.doc/SyntaxKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    // ---- Token kinds (mirror `TokenKind`) ----
    #[doc = include_str!("syntax_kind.doc/LAngle.md")]
    LAngle,
    #[doc = include_str!("syntax_kind.doc/RAngle.md")]
    RAngle,
    #[doc = include_str!("syntax_kind.doc/Slash.md")]
    Slash,
    #[doc = include_str!("syntax_kind.doc/Equals.md")]
    Equals,
    #[doc = include_str!("syntax_kind.doc/LBrace.md")]
    LBrace,
    #[doc = include_str!("syntax_kind.doc/RBrace.md")]
    RBrace,
    #[doc = include_str!("syntax_kind.doc/Colon.md")]
    Colon,
    #[doc = include_str!("syntax_kind.doc/Ident.md")]
    Ident,
    #[doc = include_str!("syntax_kind.doc/String.md")]
    String,
    #[doc = include_str!("syntax_kind.doc/Number.md")]
    Number,
    #[doc = include_str!("syntax_kind.doc/Text.md")]
    Text,
    #[doc = include_str!("syntax_kind.doc/Whitespace.md")]
    Whitespace,
    #[doc = include_str!("syntax_kind.doc/Error.md")]
    Error,
    #[doc = include_str!("syntax_kind.doc/Eof.md")]
    Eof,

    // ---- Node kinds ----
    #[doc = include_str!("syntax_kind.doc/Document.md")]
    Document,
    #[doc = include_str!("syntax_kind.doc/Element.md")]
    Element,
    #[doc = include_str!("syntax_kind.doc/OpenTag.md")]
    OpenTag,
    #[doc = include_str!("syntax_kind.doc/CloseTag.md")]
    CloseTag,
    #[doc = include_str!("syntax_kind.doc/Attribute.md")]
    Attribute,
    #[doc = include_str!("syntax_kind.doc/AttributeValue.md")]
    AttributeValue,
    #[doc = include_str!("syntax_kind.doc/Expression.md")]
    Expression,
    #[doc = include_str!("syntax_kind.doc/Conditional.md")]
    Conditional,
    #[doc = include_str!("syntax_kind.doc/ForLoop.md")]
    ForLoop,
    #[doc = include_str!("syntax_kind.doc/Template.md")]
    Template,
    #[doc = include_str!("syntax_kind.doc/TextNode.md")]
    TextNode,
    #[doc = include_str!("syntax_kind.doc/ErrorNode.md")]
    ErrorNode,
}

impl SyntaxKind {
    #[doc = include_str!("syntax_kind.doc/is_token.md")]
    #[must_use]
    pub const fn is_token(&self) -> bool {
        matches!(
            self,
            Self::LAngle
                | Self::RAngle
                | Self::Slash
                | Self::Equals
                | Self::LBrace
                | Self::RBrace
                | Self::Colon
                | Self::Ident
                | Self::String
                | Self::Number
                | Self::Text
                | Self::Whitespace
                | Self::Error
                | Self::Eof
        )
    }

    #[doc = include_str!("syntax_kind.doc/is_node.md")]
    #[must_use]
    pub const fn is_node(&self) -> bool {
        !self.is_token()
    }

    #[doc = include_str!("syntax_kind.doc/is_trivia.md")]
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    #[doc = include_str!("syntax_kind.doc/from_token.md")]
    #[must_use]
    pub const fn from_token(kind: TokenKind) -> Self {
        match kind {
            TokenKind::LAngle => Self::LAngle,
            TokenKind::RAngle => Self::RAngle,
            TokenKind::Slash => Self::Slash,
            TokenKind::Equals => Self::Equals,
            TokenKind::LBrace => Self::LBrace,
            TokenKind::RBrace => Self::RBrace,
            TokenKind::Colon => Self::Colon,
            TokenKind::Ident => Self::Ident,
            TokenKind::String => Self::String,
            TokenKind::Number => Self::Number,
            TokenKind::Text => Self::Text,
            TokenKind::Whitespace => Self::Whitespace,
            TokenKind::Error => Self::Error,
            TokenKind::Eof => Self::Eof,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_token_maps_every_token_kind() {
        let cases = [
            (TokenKind::LAngle, SyntaxKind::LAngle),
            (TokenKind::RAngle, SyntaxKind::RAngle),
            (TokenKind::Slash, SyntaxKind::Slash),
            (TokenKind::Equals, SyntaxKind::Equals),
            (TokenKind::LBrace, SyntaxKind::LBrace),
            (TokenKind::RBrace, SyntaxKind::RBrace),
            (TokenKind::Colon, SyntaxKind::Colon),
            (TokenKind::Ident, SyntaxKind::Ident),
            (TokenKind::String, SyntaxKind::String),
            (TokenKind::Number, SyntaxKind::Number),
            (TokenKind::Text, SyntaxKind::Text),
            (TokenKind::Whitespace, SyntaxKind::Whitespace),
            (TokenKind::Error, SyntaxKind::Error),
            (TokenKind::Eof, SyntaxKind::Eof),
        ];
        for (tk, sk) in cases {
            let mapped = SyntaxKind::from_token(tk);
            assert_eq!(mapped, sk);
            assert!(mapped.is_token());
            assert!(!mapped.is_node());
        }
    }

    #[test]
    fn node_kinds_are_nodes_not_tokens() {
        for kind in [
            SyntaxKind::Document,
            SyntaxKind::Element,
            SyntaxKind::OpenTag,
            SyntaxKind::CloseTag,
            SyntaxKind::Attribute,
            SyntaxKind::AttributeValue,
            SyntaxKind::Expression,
            SyntaxKind::Conditional,
            SyntaxKind::ForLoop,
            SyntaxKind::Template,
            SyntaxKind::TextNode,
            SyntaxKind::ErrorNode,
        ] {
            assert!(kind.is_node());
            assert!(!kind.is_token());
        }
    }

    #[test]
    fn whitespace_is_trivia() {
        assert!(SyntaxKind::Whitespace.is_trivia());
        assert!(!SyntaxKind::Ident.is_trivia());
        assert!(!SyntaxKind::Document.is_trivia());
    }
}
