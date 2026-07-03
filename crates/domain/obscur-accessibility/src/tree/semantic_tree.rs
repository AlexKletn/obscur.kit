//! The semantic accessibility tree linked to the retained UI tree (spec §7.1).

use std::collections::BTreeMap;

use obscur_identity::NodeId;

use crate::semantic::SemanticNode;

/// The semantic accessibility tree linked to the retained UI tree (spec §7.1).
///
/// Accessibility is projected through a **semantic tree** whose nodes are keyed
/// by the retained-tree [`NodeId`]s they describe. Platform adapters target UI
/// Automation, AX, AT-SPI, or web accessibility APIs from this shared structure;
/// they never touch the retained tree directly.
///
/// The tree owns a [`SemanticNode`] per id, an ordered child list per parent,
/// an optional [`root`](Self::root), and a monotonic [`version`](Self::version)
/// counter that is bumped on every structural or content mutation. The version
/// is a plain counter — never a wall-clock value — so a [`SemanticSnapshot`]
/// taken from this tree can be compared against the live version to detect
/// staleness (spec §7.1).
///
/// [`SemanticSnapshot`]: crate::tree::SemanticSnapshot
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticTree {
    /// The semantic nodes, keyed by the retained [`NodeId`] each describes.
    nodes: BTreeMap<NodeId, SemanticNode>,
    /// Ordered child id lists, keyed by parent [`NodeId`].
    children: BTreeMap<NodeId, Vec<NodeId>>,
    /// The tree root, if one has been designated.
    root: Option<NodeId>,
    /// The monotonic mutation counter, bumped on every structural or content
    /// change; used for snapshot staleness rather than any wall-clock value.
    version: u64,
}

impl Default for SemanticTree {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticTree {
    /// Creates an empty tree with no nodes, no root, and version `0`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            children: BTreeMap::new(),
            root: None,
            version: 0,
        }
    }

    /// Designates `id` as the tree [`root`](Self::root) and bumps the version.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
        self.version += 1;
    }

    /// Returns the tree root, if one has been designated.
    #[must_use]
    pub const fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Inserts `node`, keyed by its own [`node`](SemanticNode::node) id, and
    /// bumps the version (spec §7.1).
    ///
    /// When `parent` is `Some`, the node's id is appended to that parent's
    /// ordered child list. Re-inserting the same id **updates** the stored node
    /// in place; the child list is only appended to when a `parent` is supplied,
    /// so callers updating an already-linked node should pass `None` to avoid a
    /// duplicate child entry.
    pub fn insert(&mut self, node: SemanticNode, parent: Option<NodeId>) {
        let id = node.node();
        self.nodes.insert(id, node);
        if let Some(parent) = parent {
            self.children.entry(parent).or_default().push(id);
        }
        self.version += 1;
    }

    /// Returns the semantic node for `id`, if present.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&SemanticNode> {
        self.nodes.get(&id)
    }

    /// Returns the ordered child ids of `id`, or an empty slice if it has none.
    #[must_use]
    pub fn children(&self, id: NodeId) -> &[NodeId] {
        self.children.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Removes and returns the node for `id`, unlinking it from its parent's
    /// child list and dropping its own child list, then bumps the version.
    ///
    /// Returns `None` (without bumping the version) if no such node exists. If
    /// the removed node was the [`root`](Self::root), the root is cleared.
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

    /// Returns whether a node for `id` is present.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Returns the number of nodes in the tree.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the tree holds no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the current monotonic version counter.
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Iterates over every `(NodeId, &SemanticNode)` in deterministic ascending
    /// id order.
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
