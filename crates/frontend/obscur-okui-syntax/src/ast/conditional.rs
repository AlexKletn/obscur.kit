//! `Conditional`: the typed AST view of an `<If>`/`<ElseIf>`/`<Else>` element
//! (spec §2.11).

use crate::ast::{Element, Expression};
use crate::syntax::{SyntaxKind, SyntaxNode};

/// Typed AST wrapper over a [`Conditional`](SyntaxKind::Conditional) syntax node
/// — an `<If>`, `<ElseIf>` or `<Else>` control-flow element (spec §2.11).
///
/// A conditional is element-shaped, so its tag name, attributes and content are
/// read through the shared [`Element`] view; this wrapper adds the
/// control-flow-specific accessors [`keyword`](Self::keyword),
/// [`condition`](Self::condition) and [`body`](Self::body). The branch condition
/// is carried by a `cond={…}` attribute (absent on a bare `<Else>`).
#[derive(Debug, Clone, PartialEq)]
pub struct Conditional(SyntaxNode);

impl Conditional {
    /// Casts a syntax node to a `Conditional`, returning `Some` iff its kind is
    /// [`Conditional`](SyntaxKind::Conditional).
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::Conditional {
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

    /// Returns the same node viewed through the shared [`Element`] wrapper.
    ///
    /// [`Element::cast`] accepts the conditional's kind, so this never fails.
    fn as_element(&self) -> Element {
        Element::cast(self.0.clone()).expect("Conditional is element-shaped")
    }

    /// Returns the control-flow keyword — the tag name (`"If"`, `"ElseIf"` or
    /// `"Else"`).
    #[must_use]
    pub fn keyword(&self) -> Option<String> {
        self.as_element().tag_name()
    }

    /// Returns the branch condition — the value expression of the `cond`
    /// attribute (`None` for a bare `<Else>`).
    #[must_use]
    pub fn condition(&self) -> Option<Expression> {
        self.as_element().attribute("cond")?.value_expression()
    }

    /// Iterates the conditional's body — its content children (see
    /// [`Element::children`]).
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
