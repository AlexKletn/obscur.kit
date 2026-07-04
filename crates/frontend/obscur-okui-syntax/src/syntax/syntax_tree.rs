//! `SyntaxTree`: the whole parsed lossless tree.

use crate::syntax::SyntaxNode;

/// A whole parsed `.okui` document as a lossless syntax tree (spec §2.8).
///
/// The tree owns a single root [`SyntaxNode`]. Because every leaf keeps its
/// exact source slice, [`SyntaxTree::text`] reconstructs the original source
/// byte-for-byte — the losslessness guarantee that lets tooling round-trip and
/// re-print `.okui` sources without data loss.
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

        // Wrap every non-Eof token as a flat Document node.
        let mut root = SyntaxNode::new(SyntaxKind::Document);
        for token in tokens.iter().filter(|t| t.kind() != TokenKind::Eof) {
            root.push_token(SyntaxToken::from_lexer(token));
        }

        let tree = SyntaxTree::new(root);
        assert_eq!(tree.text(), src);
        assert_eq!(tree.root().kind(), SyntaxKind::Document);
    }
}
