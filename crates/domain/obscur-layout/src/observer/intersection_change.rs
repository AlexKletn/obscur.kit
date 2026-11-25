use obscur_identity::NodeId;

/// An intersection transition reported by an [`IntersectionObserver`](super::IntersectionObserver)
/// (spec §4.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntersectionChange {
    /// The observed node.
    pub node: NodeId,
    /// The fraction (`0.0..=1.0`) of the target covered by the root at the
    /// transition.
    pub ratio: f32,
    /// Whether the target is intersecting (`ratio >= threshold`) after the
    /// transition.
    pub intersecting: bool,
}

impl IntersectionChange {
    /// Constructs an intersection change event.
    #[must_use]
    pub const fn new(node: NodeId, ratio: f32, intersecting: bool) -> Self {
        Self {
            node,
            ratio,
            intersecting,
        }
    }
}
