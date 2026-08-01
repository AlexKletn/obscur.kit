#![doc = include_str!("element.doc/_module.md")]

use crate::ast::Attribute;
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("element.doc/Element.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Element(SyntaxNode);

impl Element {
    #[doc = include_str!("element.doc/cast.md")]
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if matches!(
            node.kind(),
            SyntaxKind::Element | SyntaxKind::Conditional | SyntaxKind::ForLoop
        ) {
            Some(Self(node))
        } else {
            None
        }
    }

    #[doc = include_str!("element.doc/syntax.md")]
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("element.doc/open_tag.md")]
    fn open_tag(&self) -> Option<&SyntaxNode> {
        self.0
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::OpenTag)
    }

    #[doc = include_str!("element.doc/tag_name.md")]
    #[must_use]
    pub fn tag_name(&self) -> Option<String> {
        self.open_tag()?
            .tokens()
            .find(|t| t.kind() == SyntaxKind::Ident)
            .map(|t| t.text().to_owned())
    }

    #[doc = include_str!("element.doc/is_conditional.md")]
    #[must_use]
    pub const fn is_conditional(&self) -> bool {
        matches!(self.0.kind(), SyntaxKind::Conditional)
    }

    #[doc = include_str!("element.doc/is_loop.md")]
    #[must_use]
    pub const fn is_loop(&self) -> bool {
        matches!(self.0.kind(), SyntaxKind::ForLoop)
    }

    #[doc = include_str!("element.doc/attributes.md")]
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> + '_ {
        self.open_tag()
            .into_iter()
            .flat_map(SyntaxNode::child_nodes)
            .cloned()
            .filter_map(Attribute::cast)
    }

    #[doc = include_str!("element.doc/attribute.md")]
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<Attribute> {
        self.attributes()
            .find(|attr| attr.name().as_deref() == Some(name))
    }

    #[doc = include_str!("element.doc/children.md")]
    pub fn children(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        self.0
            .child_nodes()
            .filter(|n| {
                matches!(
                    n.kind(),
                    SyntaxKind::Element
                        | SyntaxKind::Conditional
                        | SyntaxKind::ForLoop
                        | SyntaxKind::Expression
                        | SyntaxKind::TextNode
                )
            })
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[doc = include_str!("element.doc/is_self_closing.md")]
    #[must_use]
    pub fn is_self_closing(&self) -> bool {
        self.open_tag().is_some()
            && !self
                .0
                .child_nodes()
                .any(|n| n.kind() == SyntaxKind::CloseTag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Document;
    use crate::parser::Parser;

    fn first_element(src: &str) -> Element {
        let result = Parser::new().parse(src);
        let doc = Document::cast(result.tree().root().clone()).expect("Document");
        doc.elements().next().expect("one element")
    }

    #[test]
    fn reads_tag_name_attributes_and_content() {
        let el = first_element("<Button label=\"Save\" on:click=submit>Hi</Button>");
        assert_eq!(el.tag_name().as_deref(), Some("Button"));
        assert_eq!(el.attributes().count(), 2);
        assert!(!el.is_self_closing());
        assert!(el.attribute("label").is_some());
        assert!(!el.is_conditional());
        assert!(!el.is_loop());
        // One content child: the "Hi" text node.
        assert_eq!(el.children().count(), 1);
    }

    #[test]
    fn self_closing_element_has_no_close_tag() {
        let el = first_element("<Icon/>");
        assert!(el.is_self_closing());
        assert_eq!(el.children().count(), 0);
    }

    #[test]
    fn cast_rejects_non_element_nodes() {
        let result = Parser::new().parse("<A/>");
        assert!(Element::cast(result.tree().root().clone()).is_none());
    }
}
