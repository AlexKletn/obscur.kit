use obscur_identity::NodeId;
use obscur_interaction::gesture::PanRecognizer;

#[doc = include_str!("../../docs/api/runtime/pointer_sessions.doc/ActivePan.md")]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActivePan {
    pub(super) pan: PanRecognizer,
    pub(super) containers: [Option<NodeId>; 2],
    pub(super) axis: Option<usize>,
}
