#![doc = include_str!("loop_expression.doc/_module.md")]

use crate::ast::{Element, Expression};
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("loop_expression.doc/LoopExpression.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpression(SyntaxNode);

impl LoopExpression {
    #[doc = include_str!("loop_expression.doc/cast.md")]
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::ForLoop {
            Some(Self(node))
        } else {
            None
        }
    }

    #[doc = include_str!("loop_expression.doc/syntax.md")]
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("loop_expression.doc/as_element.md")]
    fn as_element(&self) -> Element {
        Element::cast(self.0.clone()).expect("ForLoop is element-shaped")
    }

    #[doc = include_str!("loop_expression.doc/each.md")]
    #[must_use]
    pub fn each(&self) -> Option<Expression> {
        self.as_element().attribute("each")?.value_expression()
    }

    #[doc = include_str!("loop_expression.doc/key.md")]
    #[must_use]
    pub fn key(&self) -> Option<Expression> {
        self.as_element().attribute("key")?.value_expression()
    }

    #[doc = include_str!("loop_expression.doc/is_virtual.md")]
    #[must_use]
    pub fn is_virtual(&self) -> bool {
        self.as_element().tag_name().as_deref() == Some("VirtualFor")
    }

    #[doc = include_str!("loop_expression.doc/body.md")]
    pub fn body(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        self.as_element().children()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn first_loop(src: &str) -> LoopExpression {
        let result = Parser::new().parse(src);
        result
            .tree()
            .root()
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::ForLoop)
            .cloned()
            .and_then(LoopExpression::cast)
            .expect("a loop")
    }

    #[test]
    fn reads_each_key_and_body() {
        let lp = first_loop("<For each={items} key={id}><Row/></For>");
        assert_eq!(lp.each().map(|e| e.text()).as_deref(), Some("items"));
        assert_eq!(lp.key().map(|e| e.text()).as_deref(), Some("id"));
        assert!(!lp.is_virtual());
        assert_eq!(lp.body().count(), 1);
    }

    #[test]
    fn virtual_for_is_virtual() {
        let lp = first_loop("<VirtualFor each={items} key={id}><Row/></VirtualFor>");
        assert!(lp.is_virtual());
    }
}
