#![doc = include_str!("semantic_snapshot.doc/_module.md")]

use std::collections::BTreeMap;

use obscur_identity::NodeId;

use crate::semantic::SemanticNode;

use super::SemanticTree;

#[doc = include_str!("semantic_snapshot.doc/SemanticSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticSnapshot {
    #[doc = include_str!("semantic_snapshot.doc/nodes.md")]
    nodes: BTreeMap<NodeId, SemanticNode>,
    #[doc = include_str!("semantic_snapshot.doc/children.md")]
    children: BTreeMap<NodeId, Vec<NodeId>>,
    #[doc = include_str!("semantic_snapshot.doc/root.md")]
    root: Option<NodeId>,
    #[doc = include_str!("semantic_snapshot.doc/version.md")]
    version: u64,
}

impl SemanticSnapshot {
    #[doc = include_str!("semantic_snapshot.doc/from_tree.md")]
    #[must_use]
    pub fn from_tree(tree: &SemanticTree) -> Self {
        let nodes: BTreeMap<NodeId, SemanticNode> =
            tree.iter().map(|(id, node)| (id, node.clone())).collect();
        let children = nodes
            .keys()
            .filter_map(|&id| {
                let list = tree.children(id);
                (!list.is_empty()).then(|| (id, list.to_vec()))
            })
            .collect();
        Self {
            nodes,
            children,
            root: tree.root(),
            version: tree.version(),
        }
    }

    #[doc = include_str!("semantic_snapshot.doc/get.md")]
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&SemanticNode> {
        self.nodes.get(&id)
    }

    #[doc = include_str!("semantic_snapshot.doc/children_2.md")]
    #[must_use]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    #[doc = include_str!("semantic_snapshot.doc/root_2.md")]
    #[must_use]
    pub const fn root(&self) -> Option<NodeId> {
        self.root
    }

    #[doc = include_str!("semantic_snapshot.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[doc = include_str!("semantic_snapshot.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("semantic_snapshot.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[doc = include_str!("semantic_snapshot.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    #[doc = include_str!("semantic_snapshot.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &SemanticNode)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }

    #[doc = include_str!("semantic_snapshot.doc/is_stale_against.md")]
    #[must_use]
    pub const fn is_stale_against(&self, current_version: u64) -> bool {
        self.version < current_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::SemanticRole;

    fn node(n: u64) -> SemanticNode {
        SemanticNode::new(NodeId::from_raw(n), SemanticRole::Button)
    }

    fn populated() -> SemanticTree {
        let mut tree = SemanticTree::new();
        tree.insert(node(1), None);
        tree.insert(node(2), Some(NodeId::from_raw(1)));
        tree.set_root(NodeId::from_raw(1));
        tree
    }

    #[test]
    fn from_tree_captures_nodes_root_and_version() {
        let tree = populated();
        let snap = SemanticSnapshot::from_tree(&tree);
        assert_eq!(snap.len(), 2);
        assert!(!snap.is_empty());
        assert_eq!(snap.root(), Some(NodeId::from_raw(1)));
        assert_eq!(snap.version(), tree.version());
        assert!(snap.contains(NodeId::from_raw(2)));
        assert_eq!(
            snap.get(NodeId::from_raw(1)).map(SemanticNode::node),
            Some(NodeId::from_raw(1))
        );
    }

    /// A snapshot is the only thing platform adapters see, so dropping the
    /// tree's links would hand every adapter a flat tree with no hierarchy left
    /// to publish.
    #[test]
    fn from_tree_captures_the_child_links() {
        let snap = SemanticSnapshot::from_tree(&populated());
        assert_eq!(snap.children(NodeId::from_raw(1)), &[NodeId::from_raw(2)]);
        assert!(snap.children(NodeId::from_raw(2)).is_empty(), "a leaf");
    }

    #[test]
    fn iter_is_ascending_id_order() {
        let snap = SemanticSnapshot::from_tree(&populated());
        let ids: Vec<u64> = snap.iter().map(|(id, _)| id.raw()).collect();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn snapshot_is_immutable_after_further_mutation() {
        let mut tree = populated();
        let snap = SemanticSnapshot::from_tree(&tree);
        let captured_version = snap.version();

        tree.insert(node(3), Some(NodeId::from_raw(1)));
        tree.remove(NodeId::from_raw(2));

        assert_eq!(snap.len(), 2);
        assert!(snap.contains(NodeId::from_raw(2)));
        assert!(!snap.contains(NodeId::from_raw(3)));
        assert_eq!(snap.version(), captured_version);
        assert_ne!(tree.version(), captured_version);
    }

    #[test]
    fn is_stale_against_compares_versions() {
        let tree = populated();
        let snap = SemanticSnapshot::from_tree(&tree);
        assert!(!snap.is_stale_against(snap.version()));
        assert!(snap.is_stale_against(snap.version() + 1));
        assert!(!snap.is_stale_against(snap.version().saturating_sub(1)));
    }
}
