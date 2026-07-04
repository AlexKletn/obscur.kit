//! `Element`: the typed AST view of an element node (spec §2.2).

use crate::ast::Attribute;
use crate::syntax::{SyntaxKind, SyntaxNode};

/// Typed AST wrapper over an element-shaped syntax node (spec §2.2).
///
/// [`cast`](Self::cast) accepts any of the three element-shaped node kinds —
/// [`Element`](SyntaxKind::Element), [`Conditional`](SyntaxKind::Conditional)
/// (`<If>`/`<ElseIf>`/`<Else>`) and [`ForLoop`](SyntaxKind::ForLoop)
/// (`<For>`/`<VirtualFor>`) — because they all share the `<Tag …>…</Tag>`
/// shape: an [`OpenTag`](SyntaxKind::OpenTag) node holding the tag name and
/// attributes, optional content children, and (unless self-closing) a
/// [`CloseTag`](SyntaxKind::CloseTag) node.
///
/// [`is_conditional`](Self::is_conditional) / [`is_loop`](Self::is_loop) recover
/// the control-flow classification; the dedicated
/// [`Conditional`](crate::ast::Conditional) / [`LoopExpression`](crate::ast::LoopExpression)
/// wrappers expose their extra accessors.
#[derive(Debug, Clone, PartialEq)]
pub struct Element(SyntaxNode);

impl Element {
    /// Casts a syntax node to an `Element`, returning `Some` iff its kind is
    /// element-shaped ([`Element`](SyntaxKind::Element),
    /// [`Conditional`](SyntaxKind::Conditional) or [`ForLoop`](SyntaxKind::ForLoop)).
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

    /// Returns the wrapped syntax node.
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    /// Returns the element's `<Tag …>` open-tag node, if present.
    fn open_tag(&self) -> Option<&SyntaxNode> {
        self.0
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::OpenTag)
    }

    /// Returns the element's tag name — the first [`Ident`](SyntaxKind::Ident)
    /// token inside the [`OpenTag`](SyntaxKind::OpenTag) node.
    #[must_use]
    pub fn tag_name(&self) -> Option<String> {
        self.open_tag()?
            .tokens()
            .find(|t| t.kind() == SyntaxKind::Ident)
            .map(|t| t.text().to_owned())
    }

    /// Returns `true` if this element is a [`Conditional`](SyntaxKind::Conditional)
    /// (`<If>`/`<ElseIf>`/`<Else>`).
    #[must_use]
    pub const fn is_conditional(&self) -> bool {
        matches!(self.0.kind(), SyntaxKind::Conditional)
    }

    /// Returns `true` if this element is a [`ForLoop`](SyntaxKind::ForLoop)
    /// (`<For>`/`<VirtualFor>`).
    #[must_use]
    pub const fn is_loop(&self) -> bool {
        matches!(self.0.kind(), SyntaxKind::ForLoop)
    }

    /// Iterates the element's attributes — the [`Attribute`](SyntaxKind::Attribute)
    /// child nodes of its [`OpenTag`](SyntaxKind::OpenTag) node.
    pub fn attributes(&self) -> impl Iterator<Item = Attribute> + '_ {
        self.open_tag()
            .into_iter()
            .flat_map(SyntaxNode::child_nodes)
            .cloned()
            .filter_map(Attribute::cast)
    }

    /// Returns the first attribute whose [`name`](Attribute::name) equals `name`.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<Attribute> {
        self.attributes()
            .find(|attr| attr.name().as_deref() == Some(name))
    }

    /// Iterates the element's content children — the direct child nodes between
    /// its open and close tags.
    ///
    /// Only content-shaped kinds are returned ([`Element`](SyntaxKind::Element),
    /// [`Conditional`](SyntaxKind::Conditional), [`ForLoop`](SyntaxKind::ForLoop),
    /// [`Expression`](SyntaxKind::Expression), [`TextNode`](SyntaxKind::TextNode));
    /// the structural [`OpenTag`](SyntaxKind::OpenTag) and
    /// [`CloseTag`](SyntaxKind::CloseTag) nodes are skipped. The nodes are cloned
    /// so the returned iterator is owned and outlives this borrow.
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

    /// Returns `true` if the element is self-closing (`<Tag …/>`): it has an
    /// [`OpenTag`](SyntaxKind::OpenTag) but no [`CloseTag`](SyntaxKind::CloseTag).
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
