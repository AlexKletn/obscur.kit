//! The retained-node arena — the sole owner and minter of UI identity.

use obscur_identity::{NodeId, VersionStamp};

use crate::node::{NodeGeneration, NodeKind, RetainedNode};

/// One addressable slot in the [`RetainedTree`] arena.
///
/// A slot carries its own authoritative [`NodeGeneration`] independent of any
/// occupant, so that after a node is removed and the slot is later reused, ids
/// minted against the previous occupant can be recognised as stale. The
/// generation is advanced both when a slot is vacated and when it is reused.
#[derive(Debug, Clone)]
struct Slot {
    /// The slot's current generation; authoritative for stale-id detection.
    generation: NodeGeneration,
    /// The live node occupying this slot, or `None` when the slot is vacant.
    node: Option<RetainedNode>,
}

/// Packs a slot `index` and `generation` into the opaque `u64` of a [`NodeId`].
///
/// The index occupies the low 32 bits and the generation the high 32 bits. This
/// packing is an arena-private encoding: callers only ever see an opaque
/// [`NodeId`] and must not depend on its layout.
const fn pack_node_id(index: u32, generation: u32) -> NodeId {
    NodeId::from_raw((u64::from(generation) << 32) | u64::from(index))
}

/// Unpacks a [`NodeId`] into its `(index, generation)` pair.
///
/// Inverse of [`pack_node_id`]. The masked/shifted halves always fit in a `u32`
/// by construction, so the conversions never fail; the `unwrap_or(0)` fallbacks
/// exist only to keep the function total and panic-free.
fn unpack_node_id(id: NodeId) -> (u32, u32) {
    let raw = id.raw();
    let index = u32::try_from(raw & u64::from(u32::MAX)).unwrap_or(0);
    let generation = u32::try_from(raw >> 32).unwrap_or(0);
    (index, generation)
}

/// A generational arena that owns every [`RetainedNode`] in a retained tree.
///
/// This is the **single owner of UI identity** (spec §1.5): it is the only thing
/// that mints [`NodeId`]s, and every node record lives here. Layout boxes, paint
/// data, accessibility nodes and backend handles do **not** live here — they live
/// in [`SecondaryMap`](super::SecondaryMap)s keyed by the `NodeId` this arena
/// mints, and never replace that identity.
///
/// Internally the arena is a `Vec` of slots plus a free-list of vacated indices.
/// Each minted `NodeId` packs the slot index together with that slot's
/// generation (see [`pack_node_id`]); on every access the encoded generation is
/// validated against the slot's current generation, so a stale `NodeId` whose
/// slot has since been reused resolves to `None` rather than aliasing a
/// different node.
///
/// This Phase-1 implementation is a self-contained `std` arena. It is designed to
/// be swappable for a `slotmap`-backed store later behind this same public API;
/// the generational packing is deliberately kept private so that swap is
/// invisible to callers.
#[derive(Debug, Clone, Default)]
pub struct RetainedTree {
    /// Backing storage; the slot at index `i` is addressed by ids with index `i`.
    slots: Vec<Slot>,
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

    /// Allocates a fresh node of the given `kind`, minting a new [`NodeId`] and
    /// returning it.
    ///
    /// A vacated slot is reused when one is available (advancing that slot's
    /// generation), otherwise a new slot is appended. The stored
    /// [`RetainedNode`] is constructed with the freshly minted id. This is a
    /// structural mutation and advances the tree [`version`](Self::version).
    ///
    /// The arena addresses at most `u32::MAX` slots; this bound is far beyond any
    /// realistic UI tree and cannot be reached in practice.
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

    /// Appends a brand-new slot at the initial generation and returns its id.
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

    /// Returns a mutable reference to the node for `id`, or `None` if the id is
    /// stale or was never valid.
    ///
    /// Note that mutating a node through this reference does not itself advance
    /// the tree version; only structural arena mutations
    /// ([`alloc`](Self::alloc), [`remove`](Self::remove),
    /// [`set_root`](Self::set_root)) do.
    #[must_use]
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut RetainedNode> {
        let (index, generation) = unpack_node_id(id);
        let slot = self.slots.get_mut(index as usize)?;
        if slot.generation.raw() != generation {
            return None;
        }
        slot.node.as_mut()
    }

    /// Removes and returns the node for `id`, vacating its slot.
    ///
    /// The slot's generation is advanced and its index pushed onto the free-list
    /// for reuse, so the removed `id` (and any copy of it) becomes stale and will
    /// resolve to `None` thereafter. Returns `None` if the id was already stale
    /// or never valid. This is a structural mutation and advances the tree
    /// [`version`](Self::version).
    ///
    /// If the removed node was the current root, the root is cleared.
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

    /// Designates `id` as the tree root.
    ///
    /// The id is stored as-is; callers are responsible for passing a live id.
    /// This is a structural mutation and advances the tree
    /// [`version`](Self::version).
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
        self.version = self.version.bump();
    }

    /// Returns the tree's current version stamp.
    ///
    /// The version advances on every structural mutation
    /// ([`alloc`](Self::alloc), [`remove`](Self::remove),
    /// [`set_root`](Self::set_root)), so two [`TreeSnapshot`](super::TreeSnapshot)s
    /// taken at different states carry different versions.
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    /// Iterates over every live `(NodeId, &RetainedNode)` in slot order.
    #[must_use]
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
        assert_eq!(first.raw() & u64::from(u32::MAX), second.raw() & u64::from(u32::MAX));
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
        assert_eq!(tree.get(id).expect("live").children(), &[NodeId::from_raw(999)]);
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
