use super::{SyntaxNode, SyntaxToken};

/// A child of a [`SyntaxNode`](super::SyntaxNode): either an interior node or a leaf token.
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxElement {
    /// An interior node.
    Node(SyntaxNode),
    /// A leaf token.
    Token(SyntaxToken),
}

impl SyntaxElement {
    /// Returns the exact source slice covered by this child.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Self::Node(node) => node.text(),
            Self::Token(token) => token.text().to_owned(),
        }
    }

    /// Returns the byte offset of this child's first token, if any.
    #[must_use]
    pub fn offset(&self) -> Option<u32> {
        match self {
            Self::Node(node) => node.offset(),
            Self::Token(token) => Some(token.offset()),
        }
    }
}
