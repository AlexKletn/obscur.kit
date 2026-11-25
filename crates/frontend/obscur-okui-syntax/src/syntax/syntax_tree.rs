//! `SyntaxTree`: the whole parsed lossless tree.

use crate::syntax::SyntaxNode;

#[doc = include_str!("../../docs/api/syntax/syntax_tree.doc/SyntaxTree.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTree {
    root: SyntaxNode,
}

impl SyntaxTree {
    /// Creates a tree from its root node.
    #[must_use]
    pub const fn new(root: SyntaxNode) -> Self {
        Self { root }
    }

    /// Returns the tree's root node.
    #[must_use]
    pub const fn root(&self) -> &SyntaxNode {
        &self.root
    }

    /// Reconstructs the full source the tree was built from.
    #[must_use]
    pub fn text(&self) -> String {
        self.root.text()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, TokenKind};
    use crate::syntax::{SyntaxKind, SyntaxToken};

    #[test]
    fn text_round_trips_a_lexed_source() {
        let src = "<Button label=\"Save\">{count}</Button>";
        let tokens = Lexer::new().tokenize(src);

        let mut root = SyntaxNode::new(SyntaxKind::Document);
        for token in tokens.iter().filter(|t| t.kind() != TokenKind::Eof) {
            root.push_token(SyntaxToken::from_lexer(token));
        }

        let tree = SyntaxTree::new(root);
        assert_eq!(tree.text(), src);
        assert_eq!(tree.root().kind(), SyntaxKind::Document);
    }
}
