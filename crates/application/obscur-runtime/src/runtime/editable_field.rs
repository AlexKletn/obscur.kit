#![doc = include_str!("editable_field.doc/_module.md")]

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::NodeId;
use obscur_tree::node::RetainedNode;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::EVENT_INPUT;

use super::{AncestorChain, SemanticTreeBuilder};

#[doc = include_str!("editable_field.doc/EditableField.md")]
pub(crate) struct EditableField;

impl EditableField {
    #[doc = include_str!("editable_field.doc/editable_ancestor.md")]
    pub(crate) fn editable_ancestor(tree: &RetainedTree, target: NodeId) -> Option<NodeId> {
        AncestorChain::of(tree, Some(target))
            .into_iter()
            .find(|&node| {
                tree.get(node).is_some_and(|n| {
                    n.command_for_event(EVENT_INPUT).is_some() && !Self::holds_its_own_value(n)
                })
            })
    }

    #[doc = include_str!("editable_field.doc/holds_its_own_value.md")]
    fn holds_its_own_value(node: &RetainedNode) -> bool {
        // `on:input` is how EVERY value-bearing control reports itself, exactly
        // as on the web — a slider, a checkbox and a field all speak through it.
        // So the binding alone cannot mean "this is text": a slider carrying one
        // would seat a caret inside itself and swallow the arrow keys that are
        // supposed to move its thumb. What the two kinds apart is the role, and
        // that is the same test the web applies to an `<input>`'s `type`.
        matches!(
            SemanticTreeBuilder::role_of(node),
            SemanticRole::Slider
                | SemanticRole::ProgressBar
                | SemanticRole::Checkbox
                | SemanticRole::RadioButton
                | SemanticRole::Button
                | SemanticRole::Link
        )
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
    use obscur_geometry::Size;
    use obscur_identity::{ComponentId, InternedSymbol, RuntimeId};
    use obscur_view::view::{
        LOCAL_NAME, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
    };

    use crate::UiRuntime;

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

    #[test]
    fn a_control_that_reports_its_own_value_is_not_a_text_field() {
        // The defect this caught: `on:input` is how EVERY value-bearing control
        // reports itself, so reading the binding as "this is text" made a slider
        // editable. The runtime then seated a caret in it on press and handed
        // the editing session the arrow keys meant to move its thumb.
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
        let node = |name: &str| {
            ViewNode::element(ComponentId::from_raw(0x00D0_0001))
                .with_prop(ViewProperty::new(LOCAL_NAME, ViewValue::Text(name.into())))
                .with_event(ViewEventBinding::new(
                    EVENT_INPUT,
                    InternedSymbol::from_raw(9),
                ))
        };
        let view = View::single(
            ViewNode::element(ComponentId::from_raw(0x00C0_0001)).with_children(
                ViewFragment::from_nodes(vec![node("input"), node("slider")]),
            ),
        );
        let _ = rt.render_now(&view, &[]);
        let root = rt.tree().root().expect("a root");
        let panel = rt.tree().get(root).expect("live").child_nodes()[0];
        let kids = rt.tree().get(panel).expect("live").child_nodes().to_vec();

        assert_eq!(
            EditableField::editable_ancestor(rt.tree(), kids[0]),
            Some(kids[0]),
            "a field reporting input is editable"
        );
        assert_eq!(
            EditableField::editable_ancestor(rt.tree(), kids[1]),
            None,
            "a slider reporting the same event is not"
        );
    }
}
