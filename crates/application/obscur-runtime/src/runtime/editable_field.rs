#![doc = include_str!("editable_field.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::EVENT_INPUT;

use super::AncestorChain;

#[doc = include_str!("editable_field.doc/EditableField.md")]
pub(crate) struct EditableField;

impl EditableField {
    #[doc = include_str!("editable_field.doc/editable_ancestor.md")]
    pub(crate) fn editable_ancestor(tree: &RetainedTree, target: NodeId) -> Option<NodeId> {
        AncestorChain::of(tree, Some(target))
            .into_iter()
            .find(|&node| {
                tree.get(node)
                    .and_then(|n| n.command_for_event(EVENT_INPUT))
                    .is_some()
            })
    }

    #[doc = include_str!("editable_field.doc/is_multiline.md")]
    pub(crate) fn is_multiline(tree: &RetainedTree, node: NodeId) -> bool {
        tree.get(node).and_then(|n| n.attributes().local_name()) == Some("textarea")
    }

    #[doc = include_str!("editable_field.doc/first_text_descendant.md")]
    pub(crate) fn first_text_descendant(tree: &RetainedTree, node: NodeId) -> Option<NodeId> {
        let n = tree.get(node)?;
        if n.kind().text().is_some() {
            return Some(node);
        }
        for &child in n.child_nodes() {
            if let Some(found) = Self::first_text_descendant(tree, child) {
                return Some(found);
            }
        }
        None
    }

    #[doc = include_str!("editable_field.doc/collect_text.md")]
    pub(crate) fn collect_text(tree: &RetainedTree, node: NodeId) -> String {
        let mut out = String::new();
        Self::collect_text_into(tree, node, &mut out);
        out
    }

    #[doc = include_str!("editable_field.doc/collect_text_into.md")]
    fn collect_text_into(tree: &RetainedTree, node: NodeId, out: &mut String) {
        let Some(n) = tree.get(node) else {
            return;
        };
        if n.attributes().has_class("placeholder") {
            return;
        }
        if let Some(text) = n.kind().text() {
            out.push_str(text);
        }
        for &child in n.child_nodes() {
            Self::collect_text_into(tree, child, out);
        }
    }

    #[doc = include_str!("editable_field.doc/declared_control_value.md")]
    pub(crate) fn declared_control_value(tree: &RetainedTree, node: NodeId) -> Option<String> {
        tree.get(node)
            .and_then(|n| n.attributes().control_value())
            .map(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_node_outside_the_tree_is_not_multiline_and_has_no_text_leaf() {
        let tree = RetainedTree::new();
        let stale = NodeId::from_raw(11);
        assert!(!EditableField::is_multiline(&tree, stale));
        assert_eq!(EditableField::first_text_descendant(&tree, stale), None);
        assert_eq!(EditableField::collect_text(&tree, stale), "");
        assert_eq!(EditableField::declared_control_value(&tree, stale), None);
        assert_eq!(EditableField::editable_ancestor(&tree, stale), None);
    }
}
