#![doc = include_str!("template.doc/_module.md")]

use crate::ast::Element;
use crate::syntax::{SyntaxKind, SyntaxNode};

#[doc = include_str!("template.doc/Template.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct Template(SyntaxNode);

impl Template {
    #[doc = include_str!("template.doc/cast.md")]
    #[must_use]
    pub fn cast(node: SyntaxNode) -> Option<Self> {
        if matches!(node.kind(), SyntaxKind::Template | SyntaxKind::Document) {
            Some(Self(node))
        } else {
            None
        }
    }

    #[doc = include_str!("template.doc/syntax.md")]
    #[must_use]
    pub const fn syntax(&self) -> &SyntaxNode {
        &self.0
    }

    #[doc = include_str!("template.doc/roots.md")]
    pub fn roots(&self) -> impl Iterator<Item = Element> + '_ {
        self.0.child_nodes().cloned().filter_map(Element::cast)
    }

    #[doc = include_str!("template.doc/slot_names.md")]
    #[must_use]
    pub fn slot_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        collect_slot_names(&self.0, &mut names);
        names
    }
}

#[doc = include_str!("template.doc/collect_slot_names.md")]
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
