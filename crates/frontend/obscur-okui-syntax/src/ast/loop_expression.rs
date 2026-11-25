//! `LoopExpression`: the typed AST view of a `<For>`/`<VirtualFor>` element
//! (spec §2.11).

use crate::ast::{Element, Expression};
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("../../docs/api/ast/loop_expression.doc/LoopExpression.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LoopExpression(SyntaxNode);

impl LoopExpression {
    /// Casts a syntax node to a `LoopExpression`, returning `Some` iff its kind
    /// is [`ForLoop`](SyntaxKind::ForLoop).
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::ForLoop {
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

    #[doc = include_str!("../../docs/api/ast/loop_expression.doc/as_element.md")]
    fn as_element(&self) -> Element {
        Element::cast(self.0.clone()).expect("ForLoop is element-shaped")
    }

    /// Returns the iterated collection — the value expression of the `each`
    /// attribute.
    #[must_use]
    pub fn each(&self) -> Option<Expression> {
        self.as_element().attribute("each")?.value_expression()
    }

    /// Returns the stable key — the value expression of the `key` attribute
    /// (spec §2.11 mandates a stable key per item).
    #[must_use]
    pub fn key(&self) -> Option<Expression> {
        self.as_element().attribute("key")?.value_expression()
    }

    /// Returns `true` if this is a virtualized loop (`<VirtualFor>`).
    #[must_use]
    pub fn is_virtual(&self) -> bool {
        self.as_element().tag_name().as_deref() == Some("VirtualFor")
    }

    /// Iterates the loop's body — its content children (see
    /// [`Element::children`]).
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
