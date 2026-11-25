use obscur_geometry::AffineTransform;
use obscur_identity::NodeId;

/// A transform change reported by a [`TransformObserver`](super::TransformObserver) (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformChange {
    /// The observed node.
    pub node: NodeId,
    /// The newly-committed transform.
    pub current: AffineTransform,
}

impl TransformChange {
    /// Constructs a transform change event.
    #[must_use]
    pub const fn new(node: NodeId, current: AffineTransform) -> Self {
        Self { node, current }
    }
}
