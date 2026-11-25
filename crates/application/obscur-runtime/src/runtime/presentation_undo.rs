//! What it takes to put the computed styles back after a frame of motion: `PresentationUndo`.

use obscur_identity::NodeId;
use obscur_style::computed::ComputedValue;
use obscur_style::property::PropertyId;

#[doc = include_str!("../../docs/api/runtime/presentation_undo.doc/PresentationUndo.md")]
#[derive(Debug, Default)]
pub(crate) struct PresentationUndo {
    /// One record per overlaid property, in the order they were written.
    pub(crate) entries: Vec<(NodeId, PropertyId, Option<ComputedValue>, bool)>,
}
