use obscur_identity::{FastMap, NodeId};

#[cfg(test)]
use super::paint_layer::PaintLayer;

#[doc = include_str!("../../docs/api/orchestration/stacking.doc/EffectiveStacking.md")]
pub(crate) struct EffectiveStacking {
    /// Each node's effective stacking `z`.
    pub(crate) z: FastMap<NodeId, i32>,
    /// Each node's paint-layer rank, keyed like `z`.
    #[cfg(test)]
    pub(crate) layer: FastMap<NodeId, PaintLayer>,
    pub(crate) paint_index: FastMap<NodeId, usize>,
    pub(crate) paint_band: FastMap<NodeId, i32>,
    pub(crate) context_end_band: FastMap<NodeId, i32>,
}
