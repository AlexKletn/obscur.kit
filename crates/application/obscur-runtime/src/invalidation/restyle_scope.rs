#![doc = include_str!("restyle_scope.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_style::invalidation::{StyleInvalidation, StyleInvalidationReason};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::PseudoStates;

#[doc = include_str!("restyle_scope.doc/RestyleScope.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct RestyleScope;

impl RestyleScope {
    #[doc = include_str!("restyle_scope.doc/changed_states.md")]
    #[must_use]
    pub fn changed_states(before: &PseudoStates, after: &PseudoStates) -> Vec<NodeId> {
        let mut changed = Vec::new();
        for (node, state) in after.iter() {
            if before.get(node) != state {
                changed.push(node);
            }
        }
        // A node present in `before` but absent from `after` lost its state
        // entirely; `after.get` canonicalises that to empty, and every entry in
        // `before` is non-empty, so an empty answer means "dropped".
        for (node, _) in before.iter() {
            if after.get(node).is_empty() {
                changed.push(node);
            }
        }
        changed
    }

    #[doc = include_str!("restyle_scope.doc/expand.md")]
    pub fn expand(tree: &RetainedTree, root: NodeId, out: &mut StyleInvalidation) -> bool {
        let Some(node) = tree.get(root) else {
            return true;
        };
        Self::mark_subtree(tree, root, out);

        let Some(parent) = node.logical_parent().parent() else {
            // No parent: no sibling can precede-sibling its way to `root`, so its
            // subtree is the whole reach. This is the tree's synthetic host node,
            // whose subtree is everything anyway.
            return true;
        };
        let Some(parent_node) = tree.get(parent) else {
            return false;
        };
        let Some(index) = parent_node.child_nodes().iter().position(|&c| c == root) else {
            return false;
        };
        // `+` and `~` read backwards, so only what follows `root` can see it.
        for &sibling in &parent_node.child_nodes()[index + 1..] {
            Self::mark_subtree(tree, sibling, out);
        }
        true
    }

    #[doc = include_str!("restyle_scope.doc/mark_subtree.md")]
    fn mark_subtree(tree: &RetainedTree, root: NodeId, out: &mut StyleInvalidation) {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if out.is_invalidated(id) {
                continue;
            }
            out.invalidate(id, StyleInvalidationReason::PseudoStateChanged);
            if let Some(node) = tree.get(id) {
                stack.extend(node.child_nodes().iter().copied());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_style::selector::PseudoState;
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// An element kind; the component identity is irrelevant to the reach rule,
    /// which is purely structural.
    fn element() -> NodeKind {
        NodeKind::Element {
            component: ComponentId::from_raw(1),
        }
    }

    /// Builds `parent -> children` links directly, so these tests exercise the
    /// reach rule without depending on the reconciler's view lowering.
    fn link(tree: &mut RetainedTree, parent: NodeId, children: &[NodeId]) {
        for &child in children {
            tree.get_mut(child)
                .unwrap()
                .set_logical_parent(LogicalParent::of(parent));
        }
        tree.get_mut(parent).unwrap().children = children.to_vec();
    }

    /// `host -> [a, b, c]`, `a -> [a1]`, `b -> [b1]`. Returns the ids in that
    /// order.
    fn tree_with_siblings() -> (RetainedTree, [NodeId; 6]) {
        let mut tree = RetainedTree::new();
        let host = tree.alloc(NodeKind::Root);
        let a = tree.alloc(element());
        let b = tree.alloc(element());
        let c = tree.alloc(element());
        let a1 = tree.alloc(element());
        let b1 = tree.alloc(element());
        tree.set_root(host);
        link(&mut tree, host, &[a, b, c]);
        link(&mut tree, a, &[a1]);
        link(&mut tree, b, &[b1]);
        (tree, [host, a, b, c, a1, b1])
    }

    fn marked(inv: &StyleInvalidation) -> Vec<NodeId> {
        let mut nodes: Vec<NodeId> = inv.iter().map(|(&node, _)| node).collect();
        nodes.sort_unstable();
        nodes
    }

    #[test]
    fn reach_is_the_subtree_plus_following_siblings_subtrees() {
        let (tree, [_host, a, b, c, a1, b1]) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(&tree, a, &mut inv));

        // `a` and `a1` (its subtree); `b`/`b1`/`c` because `.a:hover + .b`,
        // `.a:hover ~ .c` and `.a:hover + .b > .b1` are all expressible.
        let mut expected = vec![a, a1, b, b1, c];
        expected.sort_unstable();
        assert_eq!(marked(&inv), expected);
    }

    #[test]
    fn a_preceding_sibling_is_never_reached() {
        // THE point of the reach rule: `+`/`~` read backwards, so hovering `b`
        // cannot change `a`'s style — no selector can navigate from `a` to `b`.
        let (tree, [host, a, b, c, a1, b1]) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(&tree, b, &mut inv));

        let mut expected = vec![b, b1, c];
        expected.sort_unstable();
        assert_eq!(marked(&inv), expected);
        assert!(!inv.is_invalidated(a), "a precedes b");
        assert!(!inv.is_invalidated(a1), "a1 is inside a");
        assert!(!inv.is_invalidated(host), "an ancestor cannot see b");
    }

    #[test]
    fn a_leaf_reaches_only_itself() {
        let (tree, [_host, _a, _b, _c, a1, _b1]) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(&tree, a1, &mut inv));
        assert_eq!(marked(&inv), vec![a1]);
    }

    #[test]
    fn the_host_reaches_the_whole_tree() {
        let (tree, ids) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(&tree, ids[0], &mut inv));
        let mut expected = ids.to_vec();
        expected.sort_unstable();
        assert_eq!(marked(&inv), expected);
    }

    #[test]
    fn every_marked_node_carries_the_pseudo_state_reason() {
        let (tree, [_host, a, ..]) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(&tree, a, &mut inv));
        assert!(
            inv.iter()
                .all(|(_, &reason)| reason == StyleInvalidationReason::PseudoStateChanged)
        );
    }

    #[test]
    fn two_roots_union_their_reaches_without_double_marking() {
        let (tree, [_host, a, b, c, a1, b1]) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        // Hover moves from `a1` to `b1`: both ends are expanded into one set.
        assert!(RestyleScope::expand(&tree, a1, &mut inv));
        assert!(RestyleScope::expand(&tree, b1, &mut inv));
        let mut expected = vec![a1, b1];
        expected.sort_unstable();
        assert_eq!(marked(&inv), expected);
        for absent in [a, b, c] {
            assert!(!inv.is_invalidated(absent));
        }
    }

    #[test]
    fn a_cyclic_child_link_terminates() {
        let mut tree = RetainedTree::new();
        let host = tree.alloc(NodeKind::Root);
        let child = tree.alloc(element());
        tree.set_root(host);
        link(&mut tree, host, &[child]);
        // A malformed link back to the host.
        tree.get_mut(child).unwrap().children = vec![host];

        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(&tree, child, &mut inv));
        let mut expected = vec![host, child];
        expected.sort_unstable();
        assert_eq!(marked(&inv), expected);
    }

    #[test]
    fn a_node_outside_the_tree_reaches_nothing() {
        let (tree, _) = tree_with_siblings();
        let mut inv = StyleInvalidation::new();
        assert!(RestyleScope::expand(
            &tree,
            NodeId::from_raw(9999),
            &mut inv
        ));
        assert!(inv.is_empty());
    }

    #[test]
    fn a_parent_that_disowns_the_node_cannot_be_bounded() {
        let (mut tree, [host, a, b, c, ..]) = tree_with_siblings();
        // `a` still claims `host` as its logical parent, but `host` no longer
        // lists it: the following-sibling half of the reach is unknowable.
        tree.get_mut(host).unwrap().children = vec![b, c];

        let mut inv = StyleInvalidation::new();
        assert!(
            !RestyleScope::expand(&tree, a, &mut inv),
            "an unbounded scope must be reported, not guessed at"
        );
    }

    #[test]
    fn changed_states_reports_set_cleared_and_altered_nodes() {
        let (_tree, [_host, a, b, c, ..]) = tree_with_siblings();
        let mut before = PseudoStates::new();
        before.set(a, PseudoState::HOVER);
        before.set(b, PseudoState::FOCUS);
        let mut after = PseudoStates::new();
        // `a` unchanged, `b` cleared, `c` newly hovered.
        after.set(a, PseudoState::HOVER);
        after.set(c, PseudoState::HOVER);

        let mut changed = RestyleScope::changed_states(&before, &after);
        changed.sort_unstable();
        let mut expected = vec![b, c];
        expected.sort_unstable();
        assert_eq!(changed, expected);
    }

    #[test]
    fn changed_states_reports_a_node_whose_bits_only_grew() {
        let (_tree, [_host, a, ..]) = tree_with_siblings();
        let mut before = PseudoStates::new();
        before.set(a, PseudoState::HOVER);
        let mut after = PseudoStates::new();
        after.set(a, PseudoState::HOVER.insert(PseudoState::ACTIVE));
        assert_eq!(RestyleScope::changed_states(&before, &after), vec![a]);
    }

    #[test]
    fn identical_maps_change_nothing() {
        let (_tree, [_host, a, b, ..]) = tree_with_siblings();
        let mut before = PseudoStates::new();
        before.set(a, PseudoState::HOVER);
        before.set(b, PseudoState::FOCUS);
        let after = before.clone();
        assert!(RestyleScope::changed_states(&before, &after).is_empty());
    }
}
