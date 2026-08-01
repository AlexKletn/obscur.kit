#![doc = include_str!("syntax_tree.doc/_module.md")]

use crate::syntax::SyntaxNode;

#[doc = include_str!("syntax_tree.doc/SyntaxTree.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTree {
    root: SyntaxNode,
}

impl SyntaxTree {
    #[doc = include_str!("syntax_tree.doc/new.md")]
    #[must_use]
    pub const fn new(root: SyntaxNode) -> Self {
        Self { root }
    }

    #[doc = include_str!("syntax_tree.doc/root.md")]
    #[must_use]
    pub const fn root(&self) -> &SyntaxNode {
        &self.root
    }

    #[doc = include_str!("syntax_tree.doc/text.md")]
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
