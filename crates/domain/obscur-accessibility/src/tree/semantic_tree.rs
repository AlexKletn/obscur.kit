#![doc = include_str!("semantic_tree.doc/_module.md")]

use std::collections::BTreeMap;

use obscur_identity::NodeId;

use crate::semantic::SemanticNode;

#[doc = include_str!("semantic_tree.doc/SemanticTree.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticTree {
    #[doc = include_str!("semantic_tree.doc/nodes.md")]
    nodes: BTreeMap<NodeId, SemanticNode>,
    #[doc = include_str!("semantic_tree.doc/children.md")]
    children: BTreeMap<NodeId, Vec<NodeId>>,
    #[doc = include_str!("semantic_tree.doc/root.md")]
    root: Option<NodeId>,
    #[doc = include_str!("semantic_tree.doc/version.md")]
    version: u64,
}

impl Default for SemanticTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticTree {
    #[doc = include_str!("semantic_tree.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            children: BTreeMap::new(),
            root: None,
            version: 0,
        }
    }

    #[doc = include_str!("semantic_tree.doc/set_root.md")]
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
        self.version += 1;
    }

    #[doc = include_str!("semantic_tree.doc/root_2.md")]
    #[must_use]
    pub const fn root(&self) -> Option<NodeId> {
        self.root
    }

    #[doc = include_str!("semantic_tree.doc/insert.md")]
    pub fn insert(&mut self, node: SemanticNode, parent: Option<NodeId>) {
        let id = node.node();
        self.nodes.insert(id, node);
        if let Some(parent) = parent {
            self.children.entry(parent).or_default().push(id);
        }
        self.version += 1;
    }

    #[doc = include_str!("semantic_tree.doc/get.md")]
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&SemanticNode> {
        self.nodes.get(&id)
    }

    #[doc = include_str!("semantic_tree.doc/children_2.md")]
    #[must_use]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    #[doc = include_str!("semantic_tree.doc/remove.md")]
    pub fn remove(&mut self, id: NodeId) -> Option<SemanticNode> {
        let removed = self.nodes.remove(&id)?;
        self.children.remove(&id);
        for list in self.children.values_mut() {
            list.retain(|child| *child != id);
        }
        if self.root == Some(id) {
            self.root = None;
        }
        self.version += 1;
        Some(removed)
    }

    #[doc = include_str!("semantic_tree.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    #[doc = include_str!("semantic_tree.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("semantic_tree.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[doc = include_str!("semantic_tree.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    #[doc = include_str!("semantic_tree.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &SemanticNode)> {
        self.nodes.iter().map(|(id, node)| (*id, node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::SemanticRole;

    fn node(n: u64) -> SemanticNode {
        SemanticNode::new(NodeId::from_raw(n), SemanticRole::Button)
    }

    #[test]
    fn new_is_empty_at_version_zero() {
        let tree = SemanticTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.version(), 0);
        assert_eq!(tree.root(), None);
        assert_eq!(SemanticTree::default(), tree);
    }

    #[test]
    fn insert_get_and_contains() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1), None);
        assert!(tree.contains(NodeId::from_raw(1)));
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
        assert_eq!(
            tree.get(NodeId::from_raw(1)).map(SemanticNode::node),
            Some(NodeId::from_raw(1))
        );
        assert_eq!(tree.get(NodeId::from_raw(2)), None);
    }

    #[test]
    fn insert_appends_child_to_parent() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1), None);
        tree.insert(node(2), Some(NodeId::from_raw(1)));
        tree.insert(node(3), Some(NodeId::from_raw(1)));
        assert_eq!(
            tree.children(NodeId::from_raw(1)),
            &[NodeId::from_raw(2), NodeId::from_raw(3)]
        );
        assert!(tree.children(NodeId::from_raw(2)).is_empty());
    }

    #[test]
    fn reinserting_same_id_updates_node() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1), None);
        tree.insert(node(1).with_name("updated"), None);
        assert_eq!(tree.len(), 1);
        assert_eq!(
            tree.get(NodeId::from_raw(1)).and_then(SemanticNode::name),
            Some("updated")
        );
    }

    #[test]
    fn remove_unlinks_from_parent_child_list() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1), None);
        tree.insert(node(2), Some(NodeId::from_raw(1)));
        tree.insert(node(3), Some(NodeId::from_raw(1)));

        let removed = tree.remove(NodeId::from_raw(2));
        assert_eq!(removed.map(|n| n.node()), Some(NodeId::from_raw(2)));
        assert!(!tree.contains(NodeId::from_raw(2)));
        assert_eq!(tree.children(NodeId::from_raw(1)), &[NodeId::from_raw(3)]);
    }

    #[test]
    fn remove_drops_own_children_entry_and_clears_root() {
        let mut tree = SemanticTree::new();
        tree.insert(node(1), None);
        tree.insert(node(2), Some(NodeId::from_raw(1)));
        tree.set_root(NodeId::from_raw(1));

        assert_eq!(
            tree.remove(NodeId::from_raw(1)).map(|n| n.node()),
            Some(NodeId::from_raw(1))
        );
        assert!(tree.children(NodeId::from_raw(1)).is_empty());
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn remove_missing_returns_none() {
        let mut tree = SemanticTree::new();
        assert_eq!(tree.remove(NodeId::from_raw(9)), None);
    }

    #[test]
    fn set_root_records_root() {
        let mut tree = SemanticTree::new();
        tree.set_root(NodeId::from_raw(5));
        assert_eq!(tree.root(), Some(NodeId::from_raw(5)));
    }

    #[test]
    fn version_bumps_on_every_mutation() {
        let mut tree = SemanticTree::new();
        assert_eq!(tree.version(), 0);
        tree.insert(node(1), None);
        assert_eq!(tree.version(), 1);
        tree.set_root(NodeId::from_raw(1));
        assert_eq!(tree.version(), 2);
        tree.insert(node(2), Some(NodeId::from_raw(1)));
        assert_eq!(tree.version(), 3);
        tree.remove(NodeId::from_raw(2));
        assert_eq!(tree.version(), 4);
        // Removing a missing node does not bump the version.
        tree.remove(NodeId::from_raw(99));
        assert_eq!(tree.version(), 4);
    }

    #[test]
    fn iter_is_ascending_id_order() {
        let mut tree = SemanticTree::new();
        tree.insert(node(3), None);
        tree.insert(node(1), None);
        tree.insert(node(2), None);
        let ids: Vec<u64> = tree.iter().map(|(id, _)| id.raw()).collect();
        assert_eq!(ids, [1, 2, 3]);
    }
}
