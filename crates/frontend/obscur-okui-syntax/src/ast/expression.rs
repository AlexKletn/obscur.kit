//! `Expression`: the typed AST view of an `{expr}` interpolation (spec §2.10).

use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("../../docs/api/ast/expression.doc/Expression.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Expression(SyntaxNode);

impl Expression {
    /// Casts a syntax node to an `Expression`, returning `Some` iff its kind is
    /// [`Expression`](SyntaxKind::Expression).
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Expression {
            Some(Self(node))
        } else {
            None
        }
    }

    /// Returns the wrapped syntax node.
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("../../docs/api/ast/expression.doc/text.md")]
    #[must_use]
    pub fn text(&self) -> String {
        let tokens: Vec<_> = self.0.tokens().collect();
        let mut start = 0;
        let mut end = tokens.len();
        if start < end && tokens[start].kind() == SyntaxKind::LBrace {
            start += 1;
        }
        if end > start && tokens[end - 1].kind() == SyntaxKind::RBrace {
            end -= 1;
        }
        tokens[start..end].iter().map(|t| t.text()).collect()
    }

    /// Returns `true` if the expression has no content between its braces (no
    /// non-brace tokens).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self
            .0
            .tokens()
            .any(|t| !matches!(t.kind(), SyntaxKind::LBrace | SyntaxKind::RBrace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Document, Element};
    use crate::parser::Parser;

    fn first_element(src: &str) -> Element {
        let result = Parser::new().parse(src);
        let doc = Document::cast(result.tree().root().clone()).expect("Document");
        doc.elements().next().expect("one element")
    }

    #[test]
    fn text_strips_surrounding_braces() {
        let el = first_element("<T x={a.b}/>");
        let expr = el
            .attribute("x")
            .and_then(|a| a.value_expression())
            .expect("expression");
        assert_eq!(expr.text(), "a.b");
        assert!(!expr.is_empty());
    }

    #[test]
    fn empty_expression_is_empty() {
        let el = first_element("<T x={}/>");
        let expr = el
            .attribute("x")
            .and_then(|a| a.value_expression())
            .expect("expression");
        assert!(expr.is_empty());
        assert_eq!(expr.text(), "");
    }

    #[test]
    fn cast_rejects_non_expression() {
        let result = Parser::new().parse("<A/>");
        assert!(Expression::cast(result.tree().root().clone()).is_none());
    }
}
