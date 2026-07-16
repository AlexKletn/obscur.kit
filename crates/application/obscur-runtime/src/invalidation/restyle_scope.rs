//! Turns an interaction-state change into the per-node style dirty set the style
//! phase consumes: [`RestyleScope`] (Obscur Kit 2 specs §3.4 phase 5, §4.3).
//!
//! §4.3 requires that "recalculation starts at the nearest affected layout
//! context" and propagates "downward only when constraints change". For the style
//! phase the equivalent question is: when node `N`'s `:hover` toggles, whose
//! *computed style* can differ? This module answers it — soundly, from the shape
//! of the selector engine rather than from a guess — and records the answer as a
//! [`StyleInvalidation`], the typed accumulator §4.3 asks for.

use obscur_identity::NodeId;
use obscur_style::invalidation::{StyleInvalidation, StyleInvalidationReason};
use obscur_tree::tree::RetainedTree;

use crate::orchestration::PseudoStates;

/// Computes which nodes must be restyled when interaction state changes (§4.3).
///
/// # Why a structural reach, and why it is sound
///
/// The selector matcher navigates a tree **only** through
/// [`MatchElement`](obscur_style::selector::MatchElement), whose entire
/// navigation surface is `parent()` and `previous_sibling_element()`. So while
/// matching node `X`, the states the matcher can read are those of `X` itself and
/// of the nodes reachable from `X` by repeatedly taking a parent or a *preceding*
/// sibling. Inverting that relation: node `N`'s interaction state can only be read
/// while matching
///
/// * `N` itself and its descendants (they reach `N` through `parent()`), and
/// * the siblings that **follow** `N`, and their descendants (they reach `N`
///   through `previous_sibling_element()`, possibly after some `parent()` hops).
///
/// which is exactly `subtree(N) ∪ ⋃ subtree(S)` over `N`'s following siblings —
/// what [`expand`](Self::expand) marks. The trait bound is what makes this a proof
/// rather than an enumeration of today's combinators: no implementation of the
/// matcher can consult a node outside that closure, because it has no method with
/// which to reach one. `:has()` — the one selector that would reach *upward* — is
/// not modelled; it parses to
/// [`PseudoClassSelector::Other`](obscur_style::selector::PseudoClassSelector::Other),
/// which the matcher answers `false` for unconditionally.
///
/// # Inheritance is covered by construction
///
/// A restyled node can change values its descendants inherit. That needs no extra
/// pass here because the marked set is **downward-closed**: every node it contains
/// is marked together with its whole subtree, so a descendant of anything that can
/// change is already in the set.
///
/// # What it does not narrow
///
/// The set is a sound **upper** bound, not a minimal one. It does not consult
/// which pseudo-classes the installed sheets actually mention, nor
/// [`StyleDependency`](obscur_style::invalidation::StyleDependency) — the per-node
/// record of what a node's matching really depended on — because nothing populates
/// that record during matching yet. Hovering a node inside a large subtree
/// therefore still restyles that subtree even when no rule mentions `:hover`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RestyleScope;

impl RestyleScope {
    /// Returns the nodes whose recorded interaction state differs between
    /// `before` and `after`.
    ///
    /// Both maps are canonical — a node with no entry carries
    /// [`PseudoState::empty`](obscur_style::selector::PseudoState::empty) — so the
    /// diff walks only the two maps' entries (the hover/focus chain, a handful of
    /// nodes), never the tree. A node whose entry was dropped is reported just as
    /// one whose bits changed: un-hovering must restyle too.
    ///
    /// The result is a set in all but type: each changed node appears exactly once,
    /// and the order (which follows hash iteration) is not observable — [`expand`](Self::expand)
    /// unions the reaches.
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

    /// Records in `out` every node whose computed style can depend on `root`'s
    /// interaction state, returning whether the scope could be bounded at all.
    ///
    /// Returns `false` when the tree contradicts itself — `root`'s logical parent
    /// does not resolve, or does not list `root` among its children — because the
    /// following-sibling half of the reach cannot then be enumerated. A `false`
    /// answer means "no bound was proven"; the caller must restyle everything
    /// rather than trust a partial `out`.
    ///
    /// A `root` that is not in the tree marks nothing and returns `true`: no
    /// element can navigate to a node no child list mentions, so its state reaches
    /// nobody.
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
        let Some(index) = parent_node.children().iter().position(|&c| c == root) else {
            return false;
        };
        // `+` and `~` read backwards, so only what follows `root` can see it.
        for &sibling in &parent_node.children()[index + 1..] {
            Self::mark_subtree(tree, sibling, out);
        }
        true
    }

    /// Marks `root` and every node beneath it as pending a restyle.
    ///
    /// `out` doubles as the visited set: a node already recorded was recorded by a
    /// previous `mark_subtree`, which by definition also marked its whole subtree,
    /// so skipping it skips work that is provably already done. That also makes the
    /// walk cycle-safe — a malformed child link terminates instead of looping.
    fn mark_subtree(tree: &RetainedTree, root: NodeId, out: &mut StyleInvalidation) {
        let mut stack = vec![root];
        while let Some(id) = stack.pop() {
            if out.is_invalidated(id) {
                continue;
            }
            out.invalidate(id, StyleInvalidationReason::PseudoStateChanged);
            if let Some(node) = tree.get(id) {
                stack.extend(node.children().iter().copied());
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
