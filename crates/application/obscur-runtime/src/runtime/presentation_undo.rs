#![doc = include_str!("presentation_undo.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_style::computed::ComputedValue;
use obscur_style::property::PropertyId;

#[doc = include_str!("presentation_undo.doc/PresentationUndo.md")]
#[derive(Debug, Default)]
pub(crate) struct PresentationUndo {
    #[doc = include_str!("presentation_undo.doc/entries.md")]
    pub(crate) entries: Vec<(NodeId, PropertyId, Option<ComputedValue>, bool)>,
}
