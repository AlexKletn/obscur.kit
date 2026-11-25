use obscur_identity::NodeId;

use super::paint_layer::PaintLayer;

#[derive(Clone, Copy)]
pub(super) struct ContextEntry {
    pub(super) node: NodeId,
    pub(super) z: i32,
    pub(super) layer: PaintLayer,
    pub(super) index: usize,
    pub(super) context: bool,
}

pub(super) fn context_entry_key(entry: &ContextEntry) -> (i32, PaintLayer, usize) {
    (entry.z, entry.layer, entry.index)
}
