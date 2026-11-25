//! `SyntaxKind`: the kind of every node or token in the lossless tree.

use crate::lexer::TokenKind;

#[doc = include_str!("../../docs/api/syntax/syntax_kind.doc/SyntaxKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    // ---- Token kinds (mirror `TokenKind`) ----
    /// `<` — see [`TokenKind::LAngle`].
    LAngle,
    /// `>` — see [`TokenKind::RAngle`].
    RAngle,
    /// `/` — see [`TokenKind::Slash`].
    Slash,
    /// `=` — see [`TokenKind::Equals`].
    Equals,
    /// `{` — see [`TokenKind::LBrace`].
    LBrace,
    /// `}` — see [`TokenKind::RBrace`].
    RBrace,
    /// `:` — see [`TokenKind::Colon`].
    Colon,
    /// An identifier — see [`TokenKind::Ident`].
    Ident,
    /// A string literal — see [`TokenKind::String`].
    String,
    /// A numeric literal — see [`TokenKind::Number`].
    Number,
    /// A raw text run — see [`TokenKind::Text`].
    Text,
    /// Whitespace trivia — see [`TokenKind::Whitespace`].
    Whitespace,
    /// An unrecognized character — see [`TokenKind::Error`].
    Error,
    /// End of input — see [`TokenKind::Eof`].
    Eof,

    // ---- Node kinds ----
    /// The root node covering the whole document.
    Document,
    /// A complete element: open tag, children and close tag (or self-closing).
    Element,
    /// An opening tag `<Name …>` or self-closing `<Name … />`.
    OpenTag,
    /// A closing tag `</Name>`.
    CloseTag,
    /// A single `name=value` (or bare) attribute.
    Attribute,
    /// The value side of an attribute (a literal or `{expr}`).
    AttributeValue,
    /// An `{expr}` expression interpolation.
    Expression,
    /// An `<If cond={…}>…</If>` conditional element.
    Conditional,
    /// A `<For each={…} key={…}>…</For>` loop element.
    ForLoop,
    /// The body/children of an element or control-flow construct.
    Template,
    /// A run of text content between markup.
    TextNode,
    /// A node wrapping recovered/erroneous input.
    ErrorNode,
}

impl SyntaxKind {
    /// Returns `true` if this is a token (leaf) kind.
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

    /// Returns `true` if this is a node (interior) kind.
    #[must_use]
    pub const fn is_node(&self) -> bool {
        !self.is_token()
    }

    /// Returns `true` if this kind is trivia (currently only whitespace).
    #[must_use]
    pub const fn is_trivia(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    /// Maps a lexer [`TokenKind`] to its `SyntaxKind` counterpart.
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
