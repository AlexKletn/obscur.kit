use obscur_identity::NodeId;

/// A visibility transition reported by a [`VisibilityObserver`](super::VisibilityObserver) (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisibilityChange {
    /// The observed node.
    pub node: NodeId,
    /// The node's visibility after the transition.
    pub visible: bool,
}

impl VisibilityChange {
    /// Constructs a visibility change event.
    #[must_use]
    pub const fn new(node: NodeId, visible: bool) -> Self {
        Self { node, visible }
    }
}
