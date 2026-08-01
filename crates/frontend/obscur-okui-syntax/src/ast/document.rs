#![doc = include_str!("document.doc/_module.md")]

use crate::ast::Element;
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("document.doc/Document.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Document(SyntaxNode);

impl Document {
    #[doc = include_str!("document.doc/cast.md")]
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Document {
            Some(Self(node))
        } else {
            None
        }
    }

    #[doc = include_str!("document.doc/syntax.md")]
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("document.doc/elements.md")]
    pub fn elements(&self) -> impl Iterator<Item = Element> + '_ {
        self.0.child_nodes().cloned().filter_map(Element::cast)
    }

    #[doc = include_str!("document.doc/child_count.md")]
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
