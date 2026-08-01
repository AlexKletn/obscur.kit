#![doc = include_str!("attribute.doc/_module.md")]

use crate::ast::Expression;
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("attribute.doc/Attribute.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute(SyntaxNode);

impl Attribute {
    #[doc = include_str!("attribute.doc/cast.md")]
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Attribute {
            Some(Self(node))
        } else {
            None
        }
    }

    #[doc = include_str!("attribute.doc/syntax.md")]
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("attribute.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> Option<String> {
        self.0
            .tokens()
            .find(|t| t.kind() == SyntaxKind::Ident)
            .map(|t| t.text().to_owned())
    }

    #[doc = include_str!("attribute.doc/is_event.md")]
    #[must_use]
    pub fn is_event(&self) -> bool {
        self.0.tokens().any(|t| t.kind() == SyntaxKind::Colon)
    }

    #[doc = include_str!("attribute.doc/namespace.md")]
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

    #[doc = include_str!("attribute.doc/value_expression.md")]
    #[must_use]
    pub fn value_expression(&self) -> Option<Expression> {
        self.attribute_value()?
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::Expression)
            .cloned()
            .and_then(Expression::cast)
    }

    #[doc = include_str!("attribute.doc/string_value.md")]
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

    #[doc = include_str!("attribute.doc/is_dynamic.md")]
    #[must_use]
    pub fn is_dynamic(&self) -> bool {
        self.value_expression().is_some()
    }

    #[doc = include_str!("attribute.doc/attribute_value.md")]
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
