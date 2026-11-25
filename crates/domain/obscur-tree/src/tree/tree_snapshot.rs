//! An immutable, versioned snapshot of the retained tree for heavy subsystems.

use obscur_identity::{FastMap, NodeId, VersionStamp};

use crate::node::RetainedNode;

use super::RetainedTree;

#[doc = include_str!("../../docs/api/tree/tree_snapshot.doc/TreeSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct TreeSnapshot {
    /// Owned copies of the live node records, keyed by their stable `NodeId`.
    nodes: FastMap<NodeId, RetainedNode>,
    /// The tree root at the moment of capture, if any.
    root: Option<NodeId>,
    /// The tree version at the moment of capture.
    version: VersionStamp,
}

impl TreeSnapshot {
    #[doc = include_str!("../../docs/api/tree/tree_snapshot.doc/from_tree.md")]
    #[must_use]
    pub fn from_tree(tree: &RetainedTree) -> Self {
        let nodes = tree.iter().map(|(id, node)| (id, node.clone())).collect();
        Self {
            nodes,
            root: tree.root(),
            version: tree.version(),
        }
    }

    /// Returns the captured tree root, if any.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Returns the captured node record for `id`, if it was live at capture.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&RetainedNode> {
        self.nodes.get(&id)
    }

    /// Returns `true` if `id` was present in the tree at capture.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Returns the version stamp captured with this snapshot.
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    /// Returns the number of nodes captured in this snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the snapshot captured no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterates over every captured `(NodeId, &RetainedNode)`, in unspecified
    /// order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &RetainedNode)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeKind;

    #[test]
    fn captures_nodes_root_and_version() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let child = tree.alloc(NodeKind::Root);
        tree.set_root(root);

        let snap = tree.snapshot();
        assert_eq!(snap.len(), 2);
        assert!(!snap.is_empty());
        assert_eq!(snap.root(), Some(root));
        assert_eq!(snap.version(), tree.version());
        assert!(snap.contains(child));
        assert_eq!(snap.get(root).map(RetainedNode::id), Some(root));
    }

    #[test]
    fn snapshot_is_immutable_after_further_mutation() {
        let mut tree = RetainedTree::new();
        let a = tree.alloc(NodeKind::Root);
        let snap = tree.snapshot();
        let captured_version = snap.version();

        // Mutate the live tree after snapshotting.
        let _b = tree.alloc(NodeKind::Root);
        tree.remove(a);

        // The snapshot still reflects the captured state.
        assert_eq!(snap.len(), 1);
        assert!(snap.contains(a));
        assert_eq!(snap.version(), captured_version);
        // The live tree has moved on to a different version.
        assert_ne!(tree.version(), captured_version);
    }

    #[test]
    fn different_states_carry_different_versions() {
        let mut tree = RetainedTree::new();
        let s0 = tree.snapshot();
        tree.alloc(NodeKind::Root);
        let s1 = tree.snapshot();
        assert_ne!(s0.version(), s1.version());
    }
}
