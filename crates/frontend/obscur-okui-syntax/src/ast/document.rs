//! `Document`: the typed AST view of the lossless tree root (spec §2.2).

use crate::ast::Element;
use crate::syntax::{SyntaxKind, SyntaxNode};

/// Typed AST wrapper over a [`Document`](SyntaxKind::Document) syntax node
/// (spec §2.2).
///
/// A `Document` is the root of a parsed `.okui` source. It wraps the underlying
/// [`SyntaxNode`] by value (a cheap clone) so the typed view is owned and can be
/// passed around freely while the lossless tree stays intact underneath.
///
/// Its top-level children are elements (including the element-shaped
/// [`Conditional`](SyntaxKind::Conditional) / [`ForLoop`](SyntaxKind::ForLoop)
/// control-flow constructs), interpolated expressions and raw text. Only the
/// element-shaped children are surfaced by [`elements`](Self::elements).
#[derive(Debug, Clone, PartialEq)]
pub struct Document(SyntaxNode);

impl Document {
    /// Casts a syntax node to a `Document`, returning `Some` iff its kind is
    /// [`Document`](SyntaxKind::Document).
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Document {
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

    /// Iterates the document's top-level elements.
    ///
    /// Every element-shaped child node — a plain [`Element`](SyntaxKind::Element)
    /// as well as the element-shaped [`Conditional`](SyntaxKind::Conditional) and
    /// [`ForLoop`](SyntaxKind::ForLoop) constructs — is cast to [`Element`];
    /// non-element children (expressions, text) are skipped.
    pub fn elements(&self) -> impl Iterator<Item = Element> + '_ {
        self.0.child_nodes().cloned().filter_map(Element::cast)
    }

    /// Returns the number of direct child nodes of the document.
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.0.child_nodes().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn document(src: &str) -> Document {
        let result = Parser::new().parse(src);
        Document::cast(result.tree().root().clone()).expect("root is a Document")
    }

    #[test]
    fn cast_rejects_non_document() {
        let result = Parser::new().parse("<A/>");
        let element = result
            .tree()
            .root()
            .child_nodes()
            .next()
            .expect("an element")
            .clone();
        assert_eq!(element.kind(), SyntaxKind::Element);
        assert!(Document::cast(element).is_none());
    }

    #[test]
    fn lists_top_level_elements() {
        let doc = document("<A/><B/>");
        assert_eq!(doc.elements().count(), 2);
        let names: Vec<_> = doc.elements().filter_map(|e| e.tag_name()).collect();
        assert_eq!(names, vec!["A".to_owned(), "B".to_owned()]);
    }

    #[test]
    fn child_count_counts_all_child_nodes() {
        let doc = document("<A/><B/>");
        assert_eq!(doc.child_count(), 2);
    }
}
