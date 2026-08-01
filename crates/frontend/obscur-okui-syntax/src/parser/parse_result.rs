#![doc = include_str!("parse_result.doc/_module.md")]

use crate::parser::ParseError;
use crate::syntax::SyntaxTree;

#[doc = include_str!("parse_result.doc/ParseResult.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    tree: SyntaxTree,
    errors: Vec<ParseError>,
}

impl ParseResult {
    #[doc = include_str!("parse_result.doc/new.md")]
    #[must_use]
    pub fn new(tree: SyntaxTree, errors: Vec<ParseError>) -> Self {
        Self { tree, errors }
    }

    #[doc = include_str!("parse_result.doc/tree.md")]
    #[must_use]
    pub const fn tree(&self) -> &SyntaxTree {
        &self.tree
    }

    #[doc = include_str!("parse_result.doc/errors.md")]
    #[must_use]
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    #[doc = include_str!("parse_result.doc/is_ok.md")]
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    #[doc = include_str!("parse_result.doc/error_count.md")]
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    #[doc = include_str!("parse_result.doc/into_tree.md")]
    #[must_use]
    pub fn into_tree(self) -> SyntaxTree {
        self.tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParseErrorKind;
    use crate::syntax::{SyntaxKind, SyntaxNode};

    fn empty_tree() -> SyntaxTree {
        SyntaxTree::new(SyntaxNode::new(SyntaxKind::Document))
    }

    #[test]
    fn is_ok_true_when_no_errors() {
        let result = ParseResult::new(empty_tree(), Vec::new());
        assert!(result.is_ok());
        assert_eq!(result.error_count(), 0);
        assert!(result.errors().is_empty());
    }

    #[test]
    fn is_ok_false_when_errors_present() {
        let errors = vec![ParseError::new(ParseErrorKind::UnexpectedEof, "eof", 3)];
        let result = ParseResult::new(empty_tree(), errors);
        assert!(!result.is_ok());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn into_tree_recovers_the_tree() {
        let result = ParseResult::new(empty_tree(), Vec::new());
        let tree = result.into_tree();
        assert_eq!(tree.root().kind(), SyntaxKind::Document);
    }
}
