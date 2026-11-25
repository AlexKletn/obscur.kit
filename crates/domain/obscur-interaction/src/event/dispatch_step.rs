use obscur_identity::NodeId;

use super::EventPhase;

#[doc = include_str!("../../docs/api/event/event_dispatcher.doc/DispatchStep.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchStep {
    /// The node visited at this step.
    pub node: NodeId,
    /// The propagation phase this node is visited in.
    pub phase: EventPhase,
}

impl DispatchStep {
    /// Creates a step visiting `node` during `phase`.
    #[must_use]
    pub const fn new(node: NodeId, phase: EventPhase) -> Self {
        Self { node, phase }
    }
}
