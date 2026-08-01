#![doc = include_str!("conditional.doc/_module.md")]

use crate::ast::{Element, Expression};
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("conditional.doc/Conditional.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Conditional(SyntaxNode);

impl Conditional {
    #[doc = include_str!("conditional.doc/cast.md")]
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Conditional {
            Some(Self(node))
        } else {
            None
        }
    }

    #[doc = include_str!("conditional.doc/syntax.md")]
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("conditional.doc/as_element.md")]
    fn as_element(&self) -> Element {
        Element::cast(self.0.clone()).expect("Conditional is element-shaped")
    }

    #[doc = include_str!("conditional.doc/keyword.md")]
    #[must_use]
    pub fn keyword(&self) -> Option<String> {
        self.as_element().tag_name()
    }

    #[doc = include_str!("conditional.doc/condition.md")]
    #[must_use]
    pub fn condition(&self) -> Option<Expression> {
        self.as_element().attribute("cond")?.value_expression()
    }

    #[doc = include_str!("conditional.doc/body.md")]
    pub fn body(&self) -> impl Iterator<Item = SyntaxNode> + use<> {
        self.as_element().children()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn first_conditional(src: &str) -> Conditional {
        let result = Parser::new().parse(src);
        result
            .tree()
            .root()
            .child_nodes()
            .find(|n| n.kind() == SyntaxKind::Conditional)
            .cloned()
            .and_then(Conditional::cast)
            .expect("a conditional")
    }

    #[test]
    fn reads_keyword_condition_and_body() {
        let cond = first_conditional("<If cond={open}><A/></If>");
        assert_eq!(cond.keyword().as_deref(), Some("If"));
        assert_eq!(cond.condition().map(|e| e.text()).as_deref(), Some("open"));
        assert_eq!(cond.body().count(), 1);
    }

    #[test]
    fn bare_else_has_no_condition() {
        let cond = first_conditional("<Else><A/></Else>");
        assert_eq!(cond.keyword().as_deref(), Some("Else"));
        assert!(cond.condition().is_none());
    }
}
