//! The reserved structural states that make a control inert: [`StructuralStates`].

use obscur_identity::NodeId;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::StyleAttributes;

use super::AncestorChain;

#[doc = include_str!("../../docs/api/runtime/structural_states.doc/StructuralStates.md")]
pub(crate) struct StructuralStates<'a> {
    /// The retained tree the ancestor walk and the attribute reads run over.
    tree: &'a RetainedTree,
}

impl<'a> StructuralStates<'a> {
    /// Bundles `tree` for one question about one node.
    pub(crate) const fn of(tree: &'a RetainedTree) -> Self {
        Self { tree }
    }

    #[doc = include_str!("../../docs/api/runtime/structural_states.doc/is_within_disabled.md")]
    pub(crate) fn is_within_disabled(&self, node: NodeId) -> bool {
        self.any_ancestor(node, StyleAttributes::is_disabled)
    }

    #[doc = include_str!("../../docs/api/runtime/structural_states.doc/is_within_readonly.md")]
    pub(crate) fn is_within_readonly(&self, node: NodeId) -> bool {
        self.any_ancestor(node, StyleAttributes::is_readonly)
    }

    #[doc = include_str!("../../docs/api/runtime/structural_states.doc/any_ancestor.md")]
    fn any_ancestor(&self, node: NodeId, state: impl Fn(&StyleAttributes) -> bool) -> bool {
        AncestorChain::of(self.tree, Some(node))
            .into_iter()
            .any(|ancestor| {
                self.tree
                    .get(ancestor)
                    .is_some_and(|el| state(el.attributes()))
            })
    }
}

#[cfg(test)]
mod tests {
    use obscur_identity::ComponentId;
    use obscur_tree::node::{LogicalParent, NodeKind};
    use obscur_view::view::{DISABLED, READONLY, ViewProperty, ViewValue};

    use super::*;

    /// A `parent → child` tree whose parent carries the reserved state named by
    /// `state`. Returns the tree and the child, so the ancestor reach is what is
    /// under test rather than the node's own attributes.
    fn parent_with(state: obscur_identity::InternedSymbol) -> (RetainedTree, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let parent = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(1),
        });
        {
            let node = tree.get_mut(parent).expect("just allocated");
            node.set_logical_parent(LogicalParent::of(root));
            node.set_attributes(StyleAttributes::extract(&[ViewProperty::new(
                state,
                ViewValue::Bool(true),
            )]));
        }
        tree.get_mut(root).expect("in tree").push_child(parent);
        let child = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(2),
        });
        tree.get_mut(child)
            .expect("just allocated")
            .set_logical_parent(LogicalParent::of(parent));
        tree.get_mut(parent).expect("in tree").push_child(child);
        (tree, parent, child)
    }

    #[test]
    fn a_disabled_control_disables_everything_inside_it() {
        let (tree, parent, child) = parent_with(DISABLED);
        let states = StructuralStates::of(&tree);
        assert!(states.is_within_disabled(parent));
        // The reach is the point: a press lands on the label inside the button,
        // not on the button.
        assert!(states.is_within_disabled(child));
        // Disabled is not readonly. A disabled field is inert wholesale; the
        // mutation gate is a different question with a different answer.
        assert!(!states.is_within_readonly(child));
    }

    #[test]
    fn a_readonly_control_gates_only_the_write() {
        let (tree, parent, child) = parent_with(READONLY);
        let states = StructuralStates::of(&tree);
        assert!(states.is_within_readonly(parent));
        assert!(states.is_within_readonly(child));
        // Readonly leaves focus, caret and selection alone, so the disabled gate
        // that blocks all three must not fire.
        assert!(!states.is_within_disabled(child));
    }

    #[test]
    fn a_node_that_is_not_in_the_tree_is_neither() {
        let tree = RetainedTree::new();
        let states = StructuralStates::of(&tree);
        let stale = NodeId::from_raw(7);
        assert!(!states.is_within_disabled(stale));
        assert!(!states.is_within_readonly(stale));
    }
}
