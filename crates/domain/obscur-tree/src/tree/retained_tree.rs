//! The retained-node arena — the sole owner and minter of UI identity.

mod node_slot;

use node_slot::NodeSlot;

use obscur_identity::{NodeId, VersionStamp};

use crate::node::{NodeFlags, NodeGeneration, NodeKind, RetainedNode, VisualParent};

#[doc = include_str!("../../docs/api/tree/retained_tree.doc/pack_node_id.md")]
fn pack_node_id(index: u32, generation: u32) -> NodeId {
    NodeId::from_raw((u64::from(generation) << 32) | u64::from(index))
}

#[doc = include_str!("../../docs/api/tree/retained_tree.doc/unpack_node_id.md")]
fn unpack_node_id(id: NodeId) -> (u32, u32) {
    let raw = id.raw();
    let index = u32::try_from(raw & u64::from(u32::MAX)).unwrap_or(0);
    let generation = u32::try_from(raw >> 32).unwrap_or(0);
    (index, generation)
}

#[doc = include_str!("../../docs/api/tree/retained_tree.doc/RetainedTree.md")]
#[derive(Debug, Clone, Default)]
pub struct RetainedTree {
    /// Backing storage; the slot at index `i` is addressed by ids with index `i`.
    slots: Vec<NodeSlot>,
    portals: Vec<(NodeId, NodeId)>,
    /// Indices of vacated slots available for reuse (LIFO).
    free: Vec<u32>,
    /// Count of currently live (occupied) slots.
    live: usize,
    /// The tree root, if one has been designated.
    root: Option<NodeId>,
    /// Monotonically advancing version, bumped on every structural mutation.
    version: VersionStamp,
}

impl RetainedTree {
    /// Creates an empty arena with no nodes, no root, and the initial version.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Iterates visual children by document order, followed by projected portals in projection order.
    pub fn visual_children(&self, parent: NodeId) -> impl DoubleEndedIterator<Item = NodeId> + '_ {
        let children = self.get(parent).map(|node| node.child_nodes());
        children
            .into_iter()
            .flat_map(|children| children.iter().copied())
            .filter(move |child| !self.portals.iter().any(|(source, _)| source == child))
            .chain(self.portals.iter().filter_map(move |&(source, host)| {
                (host == parent && self.contains(source)).then_some(source)
            }))
    }

    /// Returns the active visual parent, falling back to the logical parent.
    pub fn visual_parent_of(&self, node: NodeId) -> Option<NodeId> {
        self.portals
            .iter()
            .find(|(source, _)| *source == node)
            .map(|(_, host)| *host)
            .or_else(|| self.get(node)?.logical_parent().parent())
    }

    /// Clears the visual projections while preserving document ancestry.
    pub fn clear_portals(&mut self) {
        for (source, host) in std::mem::take(&mut self.portals) {
            self.restore_portal_source(source);
            if let Some(node) = self.get_mut(host) {
                node.remove_flags(NodeFlags::PORTAL_HOST);
            }
        }
    }

    fn restore_portal_source(&mut self, source: NodeId) {
        if let Some(node) = self.get_mut(source) {
            node.set_visual_parent(
                node.logical_parent()
                    .parent()
                    .map_or_else(VisualParent::root, VisualParent::of),
            );
        }
    }

    fn detach_portal(&mut self, source: NodeId) {
        if let Some(index) = self.portals.iter().position(|(node, _)| *node == source) {
            let (_, host) = self.portals.remove(index);
            self.restore_portal_source(source);
            if !self.portals.iter().any(|(_, other)| *other == host)
                && let Some(node) = self.get_mut(host)
            {
                node.remove_flags(NodeFlags::PORTAL_HOST);
            }
        }
    }

    pub(crate) fn project_portal(&mut self, source: NodeId, host: NodeId) -> bool {
        if Some(source) == self.root || !self.contains(source) || !self.contains(host) {
            return false;
        }
        let mut ancestor = Some(host);
        for _ in 0..=self.len() {
            let Some(node) = ancestor else {
                break;
            };
            if node == source {
                return false;
            }
            ancestor = self.visual_parent_of(node);
        }
        if ancestor.is_some() {
            return false;
        }
        self.detach_portal(source);
        self.portals.push((source, host));
        self.get_mut(source)
            .unwrap()
            .set_visual_parent(VisualParent::of(host));
        self.get_mut(host)
            .unwrap()
            .insert_flags(NodeFlags::PORTAL_HOST);
        true
    }

    #[doc = include_str!("../../docs/api/tree/retained_tree.doc/alloc.md")]
    pub fn alloc(&mut self, kind: NodeKind) -> NodeId {
        let id = if let Some(index) = self.free.pop() {
            self.reuse_vacated_slot(index, kind)
        } else {
            self.push_fresh_slot(kind)
        };
        self.live += 1;
        self.version = self.version.bump();
        id
    }

    fn reuse_vacated_slot(&mut self, index: u32, kind: NodeKind) -> NodeId {
        if let Some(slot) = self.slots.get_mut(index as usize) {
            slot.generation = slot.generation.next();
            let id = pack_node_id(index, slot.generation.raw());
            slot.node = Some(RetainedNode::new(id, kind));
            id
        } else {
            // Free-list referenced a non-existent slot (should never happen);
            // fall back to appending a fresh slot rather than panicking.
            self.push_fresh_slot(kind)
        }
    }

    /// Appends a brand-new slot at the initial generation and returns its id.
    fn push_fresh_slot(&mut self, kind: NodeKind) -> NodeId {
        let index = u32::try_from(self.slots.len()).unwrap_or(u32::MAX);
        let generation = NodeGeneration::FIRST;
        let id = pack_node_id(index, generation.raw());
        self.slots.push(NodeSlot {
            generation,
            node: Some(RetainedNode::new(id, kind)),
        });
        id
    }

    /// Returns a shared reference to the node for `id`, or `None` if the id is
    /// stale (its slot was reused) or was never valid.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&RetainedNode> {
        let (index, generation) = unpack_node_id(id);
        let slot = self.slots.get(index as usize)?;
        if slot.generation.raw() != generation {
            return None;
        }
        slot.node.as_ref()
    }

    #[doc = include_str!("../../docs/api/tree/retained_tree.doc/get_mut.md")]
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut RetainedNode> {
        let (index, generation) = unpack_node_id(id);
        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation.raw() != generation {
            return None;
        }
        slot.node.as_mut()
    }

    #[doc = include_str!("../../docs/api/tree/retained_tree.doc/remove.md")]
    pub fn remove(&mut self, id: NodeId) -> Option<RetainedNode> {
        let (index, generation) = unpack_node_id(id);
        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation.raw() != generation {
            return None;
        }
        let node = slot.node.take()?;
        slot.generation = slot.generation.next();
        self.free.push(index);
        self.live -= 1;
        if self.root == Some(id) {
            self.root = None;
        }
        self.detach_portal(id);
        while let Some(&(source, _)) = self.portals.iter().find(|(_, host)| *host == id) {
            self.detach_portal(source);
        }
        self.version = self.version.bump();
        Some(node)
    }

    #[doc = include_str!("../../docs/api/tree/retained_tree.doc/sync_element_children.md")]
    pub fn sync_element_children(&mut self, parent: NodeId) {
        let Some(child_ids) = self.get(parent).map(|node| node.children.clone()) else {
            return;
        };
        let elements: Vec<NodeId> = child_ids
            .into_iter()
            .filter(|&child| self.get(child).is_some_and(|n| n.kind().is_element()))
            .collect();
        if let Some(node) = self.get_mut(parent) {
            node.element_children = elements;
        }
    }

    /// Returns `true` if `id` currently resolves to a live node.
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.get(id).is_some()
    }

    /// Returns the number of live nodes in the arena.
    #[must_use]
    pub fn len(&self) -> usize {
        self.live
    }

    /// Returns `true` if the arena contains no live nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// Returns the designated tree root, if any.
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    #[doc = include_str!("../../docs/api/tree/retained_tree.doc/set_root.md")]
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
        self.version = self.version.bump();
    }

    #[doc = include_str!("../../docs/api/tree/retained_tree.doc/version_2.md")]
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    /// Iterates over every live `(NodeId, &RetainedNode)` in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &RetainedNode)> {
        self.slots
            .iter()
            .filter_map(|slot| slot.node.as_ref().map(|node| (node.id(), node)))
    }

    /// Captures an immutable, versioned [`TreeSnapshot`](super::TreeSnapshot) of
    /// the current live nodes for hand-off to heavy subsystems (spec §1.6).
    #[must_use]
    pub fn snapshot(&self) -> super::TreeSnapshot {
        super::TreeSnapshot::from_tree(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    fn element_kind(raw: u64) -> NodeKind {
        NodeKind::Element {
            component: ComponentId::from_raw(raw),
        }
    }

    #[test]
    fn alloc_stores_and_resolves() {
        let mut tree = RetainedTree::new();
        let id = tree.alloc(element_kind(1));
        assert!(tree.contains(id));
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
        let node = tree.get(id).expect("just allocated");
        assert_eq!(node.id(), id);
        assert!(node.kind().is_element());
    }

    #[test]
    fn ids_are_distinct_across_allocs() {
        let mut tree = RetainedTree::new();
        let a = tree.alloc(NodeKind::Root);
        let b = tree.alloc(NodeKind::Root);
        assert_ne!(a, b);
    }

    #[test]
    fn remove_makes_id_stale() {
        let mut tree = RetainedTree::new();
        let id = tree.alloc(NodeKind::Root);
        let removed = tree.remove(id);
        assert!(removed.is_some());
        assert!(!tree.contains(id));
        assert_eq!(tree.get(id), None);
        assert!(tree.is_empty());
        // Double remove is a no-op.
        assert!(tree.remove(id).is_none());
    }

    #[test]
    fn reused_slot_invalidates_old_id() {
        let mut tree = RetainedTree::new();
        let first = tree.alloc(NodeKind::Root);
        assert!(tree.remove(first).is_some());
        // Next alloc reuses the freed slot (same index, higher generation).
        let second = tree.alloc(element_kind(2));
        assert_eq!(
            first.raw() & u64::from(u32::MAX),
            second.raw() & u64::from(u32::MAX)
        );
        assert_ne!(first, second);
        // The stale id must not resolve to the new occupant.
        assert!(tree.get(first).is_none());
        assert!(tree.get(second).is_some());
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn get_mut_allows_mutation_without_version_bump() {
        let mut tree = RetainedTree::new();
        let id = tree.alloc(NodeKind::Root);
        let v = tree.version();
        let node = tree.get_mut(id).expect("live");
        node.push_child(NodeId::from_raw(999));
        assert_eq!(tree.version(), v);
        assert_eq!(
            tree.get(id).expect("live").child_nodes(),
            &[NodeId::from_raw(999)]
        );
    }

    #[test]
    fn foreign_and_unset_ids_do_not_resolve() {
        let tree = RetainedTree::new();
        assert!(tree.get(NodeId::from_raw(0)).is_none());
        assert!(tree.get(NodeId::from_raw(u64::MAX)).is_none());
    }

    #[test]
    fn root_roundtrips_and_clears_on_removal() {
        let mut tree = RetainedTree::new();
        assert_eq!(tree.root(), None);
        let id = tree.alloc(NodeKind::Root);
        tree.set_root(id);
        assert_eq!(tree.root(), Some(id));
        tree.remove(id);
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn version_advances_on_structural_mutations() {
        let mut tree = RetainedTree::new();
        let v0 = tree.version();
        let id = tree.alloc(NodeKind::Root);
        let v1 = tree.version();
        assert_ne!(v0, v1);
        tree.set_root(id);
        let v2 = tree.version();
        assert_ne!(v1, v2);
        tree.remove(id);
        let v3 = tree.version();
        assert_ne!(v2, v3);
    }

    #[test]
    fn iter_yields_only_live_nodes() {
        let mut tree = RetainedTree::new();
        let a = tree.alloc(NodeKind::Root);
        let b = tree.alloc(NodeKind::Root);
        tree.remove(a);
        let ids: Vec<NodeId> = tree.iter().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![b]);
    }

    #[test]
    fn pack_unpack_roundtrips() {
        for (index, generation) in [(0u32, 1u32), (7, 3), (u32::MAX, 2), (5, u32::MAX)] {
            let id = pack_node_id(index, generation);
            assert_eq!(unpack_node_id(id), (index, generation));
        }
    }
}
