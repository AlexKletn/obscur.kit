//! Builds an AccessKit [`TreeUpdate`] from a [`SemanticSnapshot`] (§7.1).

use std::collections::{BTreeMap, BTreeSet};

use accesskit::{Node, NodeId, Tree, TreeUpdate};
use obscur_accessibility::semantic::SemanticRelation;
use obscur_accessibility::tree::SemanticSnapshot;
use obscur_identity::NodeId as DomainNodeId;

use super::NodeConverter;
use crate::error::AccessKitBridgeError;

/// Builds a full AccessKit [`TreeUpdate`] from an immutable
/// [`SemanticSnapshot`] (spec §7.1, §2.7).
///
/// The converter walks every non-ignored node in the snapshot, converts each
/// with its [`NodeConverter`], and assembles the [`TreeUpdate`] that a host
/// feeds to a platform adapter. A snapshot without a root cannot form a valid
/// AccessKit tree, so [`convert`](Self::convert) fails with
/// [`AccessKitBridgeError::MissingRoot`].
///
/// # Tree structure
///
/// A [`SemanticSnapshot`] captures only its flat node map and root — it does
/// **not** retain the parent→child links of the live [`SemanticTree`]. The
/// converter therefore reconstructs structure from each node's
/// [`SemanticRelation::Owns`] relations: an owned target becomes an AccessKit
/// child of the owner. Any non-ignored node that is neither the root nor owned
/// by another present node is attached directly to the root, so the resulting
/// tree is always fully connected as AccessKit requires.
///
/// [`SemanticTree`]: obscur_accessibility::tree::SemanticTree
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TreeUpdateConverter {
    node: NodeConverter,
}

impl TreeUpdateConverter {
    /// Creates a converter over a fresh [`NodeConverter`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            node: NodeConverter::new(),
        }
    }

    /// Returns the inner [`NodeConverter`], e.g. to map ids consistently.
    #[must_use]
    pub const fn node_converter(&self) -> &NodeConverter {
        &self.node
    }

    /// Converts `snapshot` into a full [`TreeUpdate`] focused on its root.
    ///
    /// Equivalent to [`convert_with_focus`](Self::convert_with_focus) with no
    /// explicit focus target, so focus defaults to the root as AccessKit
    /// requires when no node is focused.
    ///
    /// # Errors
    ///
    /// Returns [`AccessKitBridgeError::MissingRoot`] when the snapshot has no
    /// root node.
    pub fn convert(&self, snapshot: &SemanticSnapshot) -> Result<TreeUpdate, AccessKitBridgeError> {
        self.convert_with_focus(snapshot, None)
    }

    /// Converts `snapshot` into a full [`TreeUpdate`], focusing `focus` when it
    /// names a present, non-ignored node and falling back to the root
    /// otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`AccessKitBridgeError::MissingRoot`] when the snapshot has no
    /// root node.
    pub fn convert_with_focus(
        &self,
        snapshot: &SemanticSnapshot,
        focus: Option<DomainNodeId>,
    ) -> Result<TreeUpdate, AccessKitBridgeError> {
        let root = snapshot.root().ok_or(AccessKitBridgeError::MissingRoot)?;

        let children = Self::reconstruct_children(snapshot, root);

        let nodes: Vec<(NodeId, Node)> = snapshot
            .iter()
            .filter(|(_, node)| !node.is_ignored())
            .map(|(id, node)| {
                let child_ids = children.get(&id).map_or(&[][..], Vec::as_slice);
                (self.node.node_id(id), self.node.convert(node, child_ids))
            })
            .collect();

        let focus_id = focus
            .filter(|id| snapshot.get(*id).is_some_and(|n| !n.is_ignored()))
            .unwrap_or(root);

        Ok(TreeUpdate {
            nodes,
            tree: Some(Tree::new(self.node.node_id(root))),
            focus: self.node.node_id(focus_id),
        })
    }

    /// Rebuilds a parent→children map from the snapshot's `Owns` relations,
    /// then attaches every otherwise-parentless non-ignored node to the root.
    fn reconstruct_children(
        snapshot: &SemanticSnapshot,
        root: DomainNodeId,
    ) -> BTreeMap<DomainNodeId, Vec<DomainNodeId>> {
        let present = |id: DomainNodeId| snapshot.get(id).is_some_and(|n| !n.is_ignored());

        let mut children: BTreeMap<DomainNodeId, Vec<DomainNodeId>> = BTreeMap::new();
        let mut owned: BTreeSet<DomainNodeId> = BTreeSet::new();

        // Explicit ownership from `Owns` relations, in deterministic id order.
        for (id, node) in snapshot.iter().filter(|(id, n)| !n.is_ignored() && *id != root) {
            for relation in node.relations() {
                let SemanticRelation::Owns(target) = relation else {
                    continue;
                };
                if *target != root && *target != id && present(*target) && owned.insert(*target) {
                    children.entry(id).or_default().push(*target);
                }
            }
        }

        // Any node without an explicit owner hangs off the root.
        let orphans: Vec<DomainNodeId> = snapshot
            .iter()
            .filter(|(id, n)| !n.is_ignored() && *id != root && !owned.contains(id))
            .map(|(id, _)| id)
            .collect();
        if !orphans.is_empty() {
            children.entry(root).or_default().extend(orphans);
        }

        children
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_accessibility::semantic::{SemanticNode, SemanticRelation, SemanticRole};
    use obscur_accessibility::tree::SemanticTree;

    fn node(n: u64, role: SemanticRole) -> SemanticNode {
        SemanticNode::new(DomainNodeId::from_raw(n), role)
    }

    fn two_node_snapshot() -> SemanticSnapshot {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(node(2, SemanticRole::Button), Some(DomainNodeId::from_raw(1)));
        tree.set_root(DomainNodeId::from_raw(1));
        SemanticSnapshot::from_tree(&tree)
    }

    #[test]
    fn two_node_snapshot_yields_two_nodes_root_and_focus() {
        let update = TreeUpdateConverter::new()
            .convert(&two_node_snapshot())
            .expect("has root");
        assert_eq!(update.nodes.len(), 2);
        assert_eq!(update.tree.as_ref().map(|t| t.root), Some(NodeId(1)));
        // No explicit focus → focus defaults to the root.
        assert_eq!(update.focus, NodeId(1));
    }

    #[test]
    fn root_gets_orphan_as_child() {
        let update = TreeUpdateConverter::new()
            .convert(&two_node_snapshot())
            .expect("has root");
        let (_, root_node) = update
            .nodes
            .iter()
            .find(|(id, _)| *id == NodeId(1))
            .expect("root present");
        assert_eq!(root_node.children(), &[NodeId(2)]);
    }

    #[test]
    fn rootless_snapshot_is_missing_root() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        let snapshot = SemanticSnapshot::from_tree(&tree);
        assert_eq!(
            TreeUpdateConverter::new().convert(&snapshot),
            Err(AccessKitBridgeError::MissingRoot)
        );
    }

    #[test]
    fn ignored_nodes_are_skipped() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(
            node(2, SemanticRole::Button).decorative(true),
            Some(DomainNodeId::from_raw(1)),
        );
        tree.set_root(DomainNodeId::from_raw(1));
        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");
        assert_eq!(update.nodes.len(), 1);
    }

    #[test]
    fn explicit_focus_is_honored() {
        let update = TreeUpdateConverter::new()
            .convert_with_focus(&two_node_snapshot(), Some(DomainNodeId::from_raw(2)))
            .expect("has root");
        assert_eq!(update.focus, NodeId(2));
    }

    #[test]
    fn owns_relation_builds_nested_structure() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1, SemanticRole::Group), None);
        tree.insert(node(3, SemanticRole::ListItem), None);
        // Node 2 owns node 3.
        tree.insert(
            node(2, SemanticRole::List)
                .with_relation(SemanticRelation::Owns(DomainNodeId::from_raw(3))),
            None,
        );
        tree.set_root(DomainNodeId::from_raw(1));
        let update = TreeUpdateConverter::new()
            .convert(&SemanticSnapshot::from_tree(&tree))
            .expect("has root");

        let child_of = |id: NodeId| {
            update
                .nodes
                .iter()
                .find(|(nid, _)| *nid == id)
                .map(|(_, n)| n.children().to_vec())
                .unwrap_or_default()
        };
        // Node 3 is owned by node 2, not attached to the root.
        assert_eq!(child_of(NodeId(2)), vec![NodeId(3)]);
        assert!(!child_of(NodeId(1)).contains(&NodeId(3)));
    }
}
