//! `Reconciler`: the engine that updates the retained tree from a desired tree.

use std::collections::HashMap;

use obscur_identity::{NodeId, StableKey};
use obscur_view::view::{View, ViewFragment, ViewNode};

use crate::node::{LogicalParent, NodeKind, VisualParent};
use crate::tree::{RetainedTree, Subtree};

use super::{ChildMatcher, IdentityMatch, ReconciliationOp, ReconciliationPlan};

/// Transforms a desired declarative tree into the retained tree, preserving node
/// identity across moves via keys.
///
/// Reconciliation is exactly the desired → retained transform (spec §2.2):
/// updates are expressed as new desired state, and this engine is the sole
/// writer that turns that state into retained-tree structure. It never invents a
/// competing identity — it reuses an existing [`NodeId`] whenever a desired child
/// is compatible with an existing one (matching by key across reorders, per spec
/// §2.11), and mints a fresh id only when it must.
///
/// Only **structure**, **identity**, and **text content** are reconciled here.
/// Element props and events flow to style/components downstream and are *not*
/// stored on the retained node by this crate.
///
/// Every pass returns a [`ReconciliationPlan`] describing what it did. The engine
/// is deterministic and panic-free, and is defensive about stale ids.
#[derive(Debug, Clone, Copy, Default)]
pub struct Reconciler {
    matcher: ChildMatcher,
}

impl Reconciler {
    /// Creates a reconciler.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            matcher: ChildMatcher::new(),
        }
    }

    /// Ensures a root node exists and reconciles its children against `desired`.
    ///
    /// If the tree already has a live root it is reused; otherwise a
    /// [`NodeKind::Root`] node is allocated and designated as the root (recorded
    /// as an [`ReconciliationOp::Insert`] with the root as its own parent). The
    /// desired [`View`]'s top-level fragment then reconciles into that root's
    /// children.
    #[must_use]
    pub fn reconcile_root(&self, tree: &mut RetainedTree, desired: &View) -> ReconciliationPlan {
        let mut plan = ReconciliationPlan::new();
        let root = match tree.root() {
            Some(id) if tree.contains(id) => id,
            _ => {
                let id = tree.alloc(NodeKind::Root);
                tree.set_root(id);
                plan.push(ReconciliationOp::Insert {
                    id,
                    parent: id,
                    index: 0,
                });
                id
            }
        };
        self.reconcile_into(tree, root, desired.root().nodes(), &mut plan);
        plan
    }

    /// Reconciles the children of an existing retained `parent` against a desired
    /// fragment, updating the tree and returning the op log.
    ///
    /// If `parent` is not a live node the pass is a no-op and an empty plan is
    /// returned.
    #[must_use]
    pub fn reconcile(
        &self,
        tree: &mut RetainedTree,
        parent: NodeId,
        desired: &ViewFragment,
    ) -> ReconciliationPlan {
        let mut plan = ReconciliationPlan::new();
        self.reconcile_into(tree, parent, desired.nodes(), &mut plan);
        plan
    }

    /// Reconciles `desired` into the children of `parent`, appending ops to
    /// `plan`. Recurses into each element child. A no-op if `parent` is stale.
    fn reconcile_into(
        &self,
        tree: &mut RetainedTree,
        parent: NodeId,
        desired: &[ViewNode],
        plan: &mut ReconciliationPlan,
    ) {
        // Read the parent's current children; bail out defensively if stale.
        let Some(existing) = tree.get(parent).map(|node| node.children().to_vec()) else {
            return;
        };

        let matching = self.matcher.match_children(tree, &existing, desired);
        let (matches, removed) = matching.into_parts();

        // Old relative order of the reused nodes, for reorder detection.
        let old_rank: HashMap<NodeId, usize> = existing
            .iter()
            .enumerate()
            .map(|(rank, &id)| (id, rank))
            .collect();

        let mut new_children: Vec<NodeId> = Vec::with_capacity(desired.len());
        let mut reused_order: Vec<NodeId> = Vec::new();

        for (index, (node, decision)) in desired.iter().zip(matches).enumerate() {
            let id = match decision {
                IdentityMatch::Reuse(id) => {
                    // Text content is the only in-place content update we apply.
                    if let ViewNode::Text { content, .. } = node {
                        if apply_text_content(tree, id, content) {
                            plan.push(ReconciliationOp::Update { id });
                        }
                    }
                    reused_order.push(id);
                    id
                }
                IdentityMatch::Create => {
                    let id = tree.alloc(node_kind_of(node));
                    plan.push(ReconciliationOp::Insert { id, parent, index });
                    id
                }
            };

            apply_identity(tree, id, parent, desired_key(node));

            // Recurse into element children (created nodes start empty, so all
            // of their desired children become inserts).
            if let ViewNode::Element { children, .. } = node {
                self.reconcile_into(tree, id, children.nodes(), plan);
            }

            new_children.push(id);
        }

        // Record moves for reused nodes whose relative order changed. Nodes that
        // stay in increasing old-rank order are left in place (no-op); the rest
        // are repositioned to their final index.
        let mut last_rank: Option<usize> = None;
        for &id in &reused_order {
            let Some(&rank) = old_rank.get(&id) else {
                continue;
            };
            let in_order = last_rank.is_none_or(|last| rank > last);
            if in_order {
                last_rank = Some(rank);
            } else if let Some(index) = new_children.iter().position(|&c| c == id) {
                plan.push(ReconciliationOp::Move { id, index });
            }
        }

        // Commit the parent's child order to the desired order.
        if let Some(parent_node) = tree.get_mut(parent) {
            parent_node.children = new_children;
        }

        // Remove every unmatched existing child together with its subtree,
        // leaves before parents so a parent is never removed before its children.
        for &root in &removed {
            let mut ids = Subtree::new(root).descendants(tree);
            ids.reverse();
            for id in ids {
                if tree.remove(id).is_some() {
                    plan.push(ReconciliationOp::Remove { id });
                }
            }
        }
    }
}

/// Maps a desired node to the retained [`NodeKind`] it materializes as.
fn node_kind_of(node: &ViewNode) -> NodeKind {
    match node {
        ViewNode::Element { component, .. } => NodeKind::Element {
            component: *component,
        },
        ViewNode::Text { content, .. } => NodeKind::Text {
            content: content.clone(),
        },
    }
}

/// Extracts the retained key (a [`StableKey`]) a desired node carries, if any.
fn desired_key(node: &ViewNode) -> Option<StableKey> {
    node.key().map(|key| key.stable_key().clone())
}

/// Sets a node's logical and visual parent to `parent` and its reconciliation
/// key. A no-op if `id` is stale.
fn apply_identity(tree: &mut RetainedTree, id: NodeId, parent: NodeId, key: Option<StableKey>) {
    if let Some(node) = tree.get_mut(id) {
        node.set_logical_parent(LogicalParent::of(parent));
        node.set_visual_parent(VisualParent::of(parent));
        node.set_key(key);
    }
}

/// Updates a reused text node's content in place, returning `true` if the
/// content actually changed. A no-op (returning `false`) if `id` is stale or is
/// not a text node.
fn apply_text_content(tree: &mut RetainedTree, id: NodeId, content: &str) -> bool {
    let Some(node) = tree.get_mut(id) else {
        return false;
    };
    // Resolve whether the content differs before taking a mutable borrow of the
    // node's kind, so the read borrow is released before the write.
    let changed = match node.kind() {
        NodeKind::Text { content: current } => current.as_ref() != content,
        _ => false,
    };
    if changed {
        node.kind = NodeKind::Text {
            content: content.into(),
        };
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_view::view::ViewKey;

    fn el(comp: u64) -> ViewNode {
        ViewNode::element(ComponentId::from_raw(comp))
    }

    fn keyed_el(comp: u64, key: &str) -> ViewNode {
        el(comp).with_key(ViewKey::from(key))
    }

    fn child_ids(tree: &RetainedTree, parent: NodeId) -> Vec<NodeId> {
        tree.get(parent).map(|n| n.children().to_vec()).unwrap_or_default()
    }

    /// Builds a tree with a root node and returns it plus the root id.
    fn tree_with_root() -> (RetainedTree, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        (tree, root)
    }

    #[test]
    fn reconcile_root_creates_root_when_missing() {
        let mut tree = RetainedTree::new();
        let view = View::single(el(1));
        let plan = Reconciler::new().reconcile_root(&mut tree, &view);
        let root = tree.root().expect("root created");
        assert!(tree.get(root).expect("live").kind().is_root());
        assert_eq!(child_ids(&tree, root).len(), 1);
        // A root insert plus one child insert.
        assert_eq!(plan.iter().filter(|op| op.is_insert()).count(), 2);
    }

    #[test]
    fn initial_children_are_all_inserts() {
        let (mut tree, root) = tree_with_root();
        let fragment = ViewFragment::from_nodes(vec![el(1), el(2), ViewNode::text("hi")]);
        let plan = Reconciler::new().reconcile(&mut tree, root, &fragment);
        assert_eq!(child_ids(&tree, root).len(), 3);
        assert_eq!(plan.iter().filter(|op| op.is_insert()).count(), 3);
        assert!(plan.iter().all(|op| op.is_insert()));
    }

    #[test]
    fn parents_and_keys_are_set_on_children() {
        let (mut tree, root) = tree_with_root();
        let fragment = ViewFragment::from_nodes(vec![keyed_el(1, "a")]);
        Reconciler::new().reconcile(&mut tree, root, &fragment);
        let child = child_ids(&tree, root)[0];
        let node = tree.get(child).expect("live");
        assert_eq!(node.logical_parent(), LogicalParent::of(root));
        assert_eq!(node.visual_parent(), VisualParent::of(root));
        assert_eq!(node.key(), Some(&StableKey::from("a")));
    }

    #[test]
    fn reorder_of_keyed_list_preserves_identity() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();

        // Initial keyed list a, b, c.
        let first = ViewFragment::from_nodes(vec![
            keyed_el(1, "a"),
            keyed_el(2, "b"),
            keyed_el(3, "c"),
        ]);
        recon.reconcile(&mut tree, root, &first);
        let ids_before = child_ids(&tree, root);
        assert_eq!(ids_before.len(), 3);
        let (a, b, c) = (ids_before[0], ids_before[1], ids_before[2]);

        // Reorder to c, a, b — same keys, different order.
        let reordered = ViewFragment::from_nodes(vec![
            keyed_el(3, "c"),
            keyed_el(1, "a"),
            keyed_el(2, "b"),
        ]);
        let plan = recon.reconcile(&mut tree, root, &reordered);

        // Identity is preserved: the very same ids, now in the desired order.
        assert_eq!(child_ids(&tree, root), vec![c, a, b]);
        assert!(tree.contains(a) && tree.contains(b) && tree.contains(c));

        // The core property: a pure reorder produces only Move ops — no
        // Insert and no Remove touch identity.
        assert!(
            plan.iter().all(|op| op.is_move()),
            "reorder must emit only Move ops, got {:?}",
            plan.ops()
        );
        assert!(plan.iter().any(|op| op.is_move()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn unchanged_reconcile_is_a_no_op() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        let fragment = ViewFragment::from_nodes(vec![keyed_el(1, "a"), keyed_el(2, "b")]);
        recon.reconcile(&mut tree, root, &fragment);
        let ids = child_ids(&tree, root);
        let plan = recon.reconcile(&mut tree, root, &fragment);
        assert_eq!(child_ids(&tree, root), ids);
        assert!(plan.is_empty(), "unchanged pass must be a no-op, got {:?}", plan.ops());
    }

    #[test]
    fn removed_child_and_its_subtree_are_removed() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();

        // parent element 'a' with a grandchild; sibling 'b'.
        let with_subtree = ViewFragment::from_nodes(vec![
            el(1)
                .with_key(ViewKey::from("a"))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text("deep")])),
            keyed_el(2, "b"),
        ]);
        recon.reconcile(&mut tree, root, &with_subtree);
        let ids = child_ids(&tree, root);
        let a = ids[0];
        let grandchild = child_ids(&tree, a)[0];
        let b = ids[1];
        assert!(tree.contains(grandchild));

        // Drop 'a' entirely; keep 'b'.
        let only_b = ViewFragment::from_nodes(vec![keyed_el(2, "b")]);
        let plan = recon.reconcile(&mut tree, root, &only_b);

        assert_eq!(child_ids(&tree, root), vec![b]);
        assert!(!tree.contains(a));
        assert!(!tree.contains(grandchild), "subtree must be removed too");
        // Both the parent and its grandchild are reported removed.
        let removed: Vec<NodeId> = plan
            .iter()
            .filter_map(|op| match op {
                ReconciliationOp::Remove { id } => Some(*id),
                _ => None,
            })
            .collect();
        assert!(removed.contains(&a));
        assert!(removed.contains(&grandchild));
    }

    #[test]
    fn incompatible_child_is_replaced_not_mutated() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![keyed_el(1, "a")]),
        );
        let old = child_ids(&tree, root)[0];

        // Same key, different component -> replace (new id, old removed).
        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![keyed_el(9, "a")]),
        );
        let new = child_ids(&tree, root)[0];
        assert_ne!(new, old);
        assert!(!tree.contains(old));
        assert!(plan.iter().any(|op| op.is_insert()));
        assert!(plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn changed_text_content_is_updated_in_place() {
        let (mut tree, root) = tree_with_root();
        let recon = Reconciler::new();
        recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![ViewNode::text("before").with_key(ViewKey::from("t"))]),
        );
        let id = child_ids(&tree, root)[0];

        let plan = recon.reconcile(
            &mut tree,
            root,
            &ViewFragment::from_nodes(vec![ViewNode::text("after").with_key(ViewKey::from("t"))]),
        );
        // Same node, updated content, one Update op.
        assert_eq!(child_ids(&tree, root)[0], id);
        assert_eq!(tree.get(id).expect("live").kind().text(), Some("after"));
        assert!(plan.iter().any(|op| op.is_update()));
        assert!(!plan.iter().any(|op| op.is_insert()));
        assert!(!plan.iter().any(|op| op.is_remove()));
    }

    #[test]
    fn stale_parent_is_a_no_op() {
        let mut tree = RetainedTree::new();
        let ghost = tree.alloc(NodeKind::Root);
        tree.remove(ghost);
        let plan = Reconciler::new().reconcile(
            &mut tree,
            ghost,
            &ViewFragment::from_nodes(vec![el(1)]),
        );
        assert!(plan.is_empty());
    }
}
