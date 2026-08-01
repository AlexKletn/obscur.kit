#![doc = include_str!("retained_tree.doc/_module.md")]

use obscur_identity::{NodeId, VersionStamp};

use crate::node::{NodeGeneration, NodeKind, RetainedNode};

#[doc = include_str!("retained_tree.doc/Slot.md")]
#[derive(Debug, Clone)]
struct Slot {
    #[doc = include_str!("retained_tree.doc/generation.md")]
    generation: NodeGeneration,
    #[doc = include_str!("retained_tree.doc/node.md")]
    node: Option<RetainedNode>,
}

#[doc = include_str!("retained_tree.doc/pack_node_id.md")]
fn pack_node_id(index: u32, generation: u32) -> NodeId {
    NodeId::from_raw((u64::from(generation) << 32) | u64::from(index))
}

#[doc = include_str!("retained_tree.doc/unpack_node_id.md")]
fn unpack_node_id(id: NodeId) -> (u32, u32) {
    let raw = id.raw();
    let index = u32::try_from(raw & u64::from(u32::MAX)).unwrap_or(0);
    let generation = u32::try_from(raw >> 32).unwrap_or(0);
    (index, generation)
}

#[doc = include_str!("retained_tree.doc/RetainedTree.md")]
#[derive(Debug, Clone, Default)]
pub struct RetainedTree {
    #[doc = include_str!("retained_tree.doc/slots.md")]
    slots: Vec<Slot>,
    #[doc = include_str!("retained_tree.doc/free.md")]
    free: Vec<u32>,
    #[doc = include_str!("retained_tree.doc/live.md")]
    live: usize,
    #[doc = include_str!("retained_tree.doc/root.md")]
    root: Option<NodeId>,
    #[doc = include_str!("retained_tree.doc/version.md")]
    version: VersionStamp,
}

impl RetainedTree {
    #[doc = include_str!("retained_tree.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("retained_tree.doc/alloc.md")]
    pub fn alloc(&mut self, kind: NodeKind) -> NodeId {
        let id = if let Some(index) = self.free.pop() {
            // Reuse a vacated slot, advancing its generation on reuse.
            let idx = index as usize;
            if let Some(slot) = self.slots.get_mut(idx) {
                slot.generation = slot.generation.next();
                let id = pack_node_id(index, slot.generation.raw());
                slot.node = Some(RetainedNode::new(id, kind));
                id
            } else {
                // Free-list referenced a non-existent slot (should never happen);
                // fall back to appending a fresh slot rather than panicking.
                self.push_fresh_slot(kind)
            }
        } else {
            self.push_fresh_slot(kind)
        };
        self.live += 1;
        self.version = self.version.bump();
        id
    }

    #[doc = include_str!("retained_tree.doc/push_fresh_slot.md")]
    fn push_fresh_slot(&mut self, kind: NodeKind) -> NodeId {
        let index = u32::try_from(self.slots.len()).unwrap_or(u32::MAX);
        let generation = NodeGeneration::FIRST;
        let id = pack_node_id(index, generation.raw());
        self.slots.push(Slot {
            generation,
            node: Some(RetainedNode::new(id, kind)),
        });
        id
    }

    #[doc = include_str!("retained_tree.doc/get.md")]
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&RetainedNode> {
        let (index, generation) = unpack_node_id(id);
        let slot = self.slots.get(index as usize)?;
        if slot.generation.raw() != generation {
            return None;
        }
        slot.node.as_ref()
    }

    #[doc = include_str!("retained_tree.doc/get_mut.md")]
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut RetainedNode> {
        let (index, generation) = unpack_node_id(id);
        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation.raw() != generation {
            return None;
        }
        slot.node.as_mut()
    }

    #[doc = include_str!("retained_tree.doc/remove.md")]
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
        self.version = self.version.bump();
        Some(node)
    }

    #[doc = include_str!("retained_tree.doc/sync_element_children.md")]
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

    #[doc = include_str!("retained_tree.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.get(id).is_some()
    }

    #[doc = include_str!("retained_tree.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.live
    }

    #[doc = include_str!("retained_tree.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    #[doc = include_str!("retained_tree.doc/root_2.md")]
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    #[doc = include_str!("retained_tree.doc/set_root.md")]
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
        self.version = self.version.bump();
    }

    #[doc = include_str!("retained_tree.doc/version_2.md")]
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    #[doc = include_str!("retained_tree.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &RetainedNode)> {
        self.slots
            .iter()
            .filter_map(|slot| slot.node.as_ref().map(|node| (node.id(), node)))
    }

    #[doc = include_str!("retained_tree.doc/snapshot.md")]
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
