//! `Template`: the typed AST view of a reusable template subtree (spec §2.12).

use crate::ast::Element;
use crate::syntax::{SyntaxKind, SyntaxNode};

/// Typed AST wrapper over a reusable template body (spec §2.12).
///
/// [`cast`](Self::cast) accepts a dedicated [`Template`](SyntaxKind::Template)
/// node as well as a whole [`Document`](SyntaxKind::Document): because the parser
/// emits `Template` nodes only rarely, an entire document can be treated as a
/// template body. Its top-level [`Element`]s are its [`roots`](Self::roots).
///
/// Named slots (§2.12) are marked with a `slot="name"` attribute on an element;
/// [`slot_names`](Self::slot_names) scans all descendants and collects those
/// names.
#[derive(Debug, Clone, PartialEq)]
pub struct Template(SyntaxNode);

impl Template {
    /// Casts a syntax node to a `Template`, returning `Some` iff its kind is
    /// [`Template`](SyntaxKind::Template) or [`Document`](SyntaxKind::Document).
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if matches!(node.kind(), SyntaxKind::Template | SyntaxKind::Document) {
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

    /// Iterates the template's top-level elements (its roots).
    pub fn roots(&self) -> impl Iterator<Item = Element> + '_ {
        self.0.child_nodes().cloned().filter_map(Element::cast)
    }

    /// Collects the names of all named slots in the template.
    ///
    /// A named slot is any descendant element carrying a `slot="name"`
    /// attribute; this recursively scans the subtree and gathers each such name
    /// in document order.
    #[must_use]
    pub fn slot_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        collect_slot_names(&self.0, &mut names);
        names
    }
}

/// Recursively collects `slot="name"` attribute values from `node` and its
/// descendants into `out`.
fn collect_slot_names(node: &SyntaxNode, out: &mut Vec<String>) {
    if let Some(element) = Element::cast(node.clone())
        && let Some(name) = element.attribute("slot").and_then(|a| a.string_value())
    {
        out.push(name);
    }
    for child in node.child_nodes() {
        collect_slot_names(child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn template(src: &str) -> Template {
        let result = Parser::new().parse(src);
        Template::cast(result.tree().root().clone()).expect("document as template")
    }

    #[test]
    fn collects_named_slots_from_descendants() {
        let tpl = template("<Panel><Slot slot=\"header\"/></Panel>");
        assert!(tpl.slot_names().contains(&"header".to_owned()));
    }

    #[test]
    fn roots_are_top_level_elements() {
        let tpl = template("<Panel><Slot slot=\"header\"/></Panel>");
        let roots: Vec<_> = tpl.roots().filter_map(|e| e.tag_name()).collect();
        assert_eq!(roots, vec!["Panel".to_owned()]);
    }

    #[test]
    fn cast_rejects_non_template_nodes() {
        let result = Parser::new().parse("<A/>");
        let element = result
            .tree()
            .root()
            .child_nodes()
            .next()
            .expect("element")
            .clone();
        assert!(Template::cast(element).is_none());
    }
}
