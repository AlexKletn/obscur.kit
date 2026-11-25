use crate::node::{NodeGeneration, RetainedNode};

#[doc = include_str!("../../../docs/api/tree/retained_tree.doc/Slot.md")]
#[derive(Debug, Clone)]
pub(super) struct NodeSlot {
    /// The slot's current generation; authoritative for stale-id detection.
    pub(super) generation: NodeGeneration,
    /// The live node occupying this slot, or `None` when the slot is vacant.
    pub(super) node: Option<RetainedNode>,
}
