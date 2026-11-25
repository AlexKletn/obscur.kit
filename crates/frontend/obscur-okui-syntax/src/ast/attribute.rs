//! `Attribute`: the typed AST view of an attribute node (spec §2.3, §2.9).

use crate::ast::Expression;
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("../../docs/api/ast/attribute.doc/Attribute.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute(SyntaxNode);

impl Attribute {
    /// Casts a syntax node to an `Attribute`, returning `Some` iff its kind is
    /// [`Attribute`](SyntaxKind::Attribute).
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Attribute {
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

    /// Returns the attribute name — the first [`Ident`](SyntaxKind::Ident) token.
    #[must_use]
    pub fn name(&self) -> Option<String> {
        self.0
            .tokens()
            .find(|t| t.kind() == SyntaxKind::Ident)
            .map(|t| t.text().to_owned())
    }

    /// Returns `true` if this is an event-style attribute — it carries a
    /// [`Colon`](SyntaxKind::Colon) token (the `prefix:name` form, e.g.
    /// `on:click`).
    #[must_use]
    pub fn is_event(&self) -> bool {
        self.0.tokens().any(|t| t.kind() == SyntaxKind::Colon)
    }

    /// Returns the namespace prefix — the [`Ident`](SyntaxKind::Ident) immediately
    /// before the [`Colon`](SyntaxKind::Colon) — for an event-style attribute
    /// (`on` in `on:click`), or `None` when the attribute has no colon.
    #[must_use]
    pub fn namespace(&self) -> Option<String> {
        let mut prev_ident: Option<&str> = None;
        for token in self.0.tokens() {
            match token.kind() {
                SyntaxKind::Ident => prev_ident = Some(token.text()),
                SyntaxKind::Colon => return prev_ident.map(str::to_owned),
                _ => {}
            }
        }
        None
    }

    /// Returns the value expression for a dynamic `attr={expr}` attribute — the
    /// [`Expression`](SyntaxKind::Expression) node inside the attribute's
    /// [`AttributeValue`](SyntaxKind::AttributeValue) node.
    #[must_use]
    pub fn value_expression(&self) -> Option<Expression> {
        self.attribute_value()?
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::Expression)
            .cloned()
            .and_then(Expression::cast)
    }

    /// Returns the literal value of a `attr="literal"` attribute — the text of
    /// the [`String`](SyntaxKind::String) token inside the
    /// [`AttributeValue`](SyntaxKind::AttributeValue) node, with its surrounding
    /// double quotes trimmed.
    #[must_use]
    pub fn string_value(&self) -> Option<String> {
        let token = self
            .attribute_value()?
            .tokens()
            .find(|t| t.kind() == SyntaxKind::String)?;
        let text = token.text();
        let text = text.strip_prefix('"').unwrap_or(text);
        let text = text.strip_suffix('"').unwrap_or(text);
        Some(text.to_owned())
    }

    /// Returns `true` if the attribute's value is a dynamic expression
    /// (`attr={expr}`) rather than a literal.
    #[must_use]
    pub fn is_dynamic(&self) -> bool {
        self.value_expression().is_some()
    }

    /// Returns the attribute's [`AttributeValue`](SyntaxKind::AttributeValue)
    /// child node, if the attribute has a value.
    fn attribute_value(&self) -> Option<&SyntaxNode> {
        self.0
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::AttributeValue)
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
    fn string_value_trims_quotes() {
        let el = first_element("<Button label=\"Save\" on:click=submit>Hi</Button>");
        let label = el.attribute("label").expect("label attribute");
        assert_eq!(label.string_value().as_deref(), Some("Save"));
        assert!(!label.is_event());
        assert!(!label.is_dynamic());
    }

    #[test]
    fn event_attribute_reports_namespace() {
        let el = first_element("<Button label=\"Save\" on:click=submit>Hi</Button>");
        let event = el
            .attributes()
            .find(Attribute::is_event)
            .expect("an event attribute");
        assert!(event.is_event());
        assert_eq!(event.namespace().as_deref(), Some("on"));
    }

    #[test]
    fn dynamic_attribute_exposes_expression() {
        let el = first_element("<T x={a.b}/>");
        let x = el.attribute("x").expect("x attribute");
        assert!(x.is_dynamic());
        assert_eq!(
            x.value_expression().map(|e| e.text()).as_deref(),
            Some("a.b")
        );
    }
}
