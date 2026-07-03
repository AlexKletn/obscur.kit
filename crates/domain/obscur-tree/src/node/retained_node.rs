//! A retained tree node (skeleton).

use obscur_identity::{Generation, NodeId};

/// A node in the retained tree. Layout, focus, animation and caches attach to
/// its stable identity (spec §2.2, §1.5). This skeleton carries only identity
/// and generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainedNode {
    /// Stable node identity.
    pub id: NodeId,
    /// Generation, bumped when the slot's identity is reused.
    pub generation: Generation,
}

impl RetainedNode {
    /// Creates a retained node at its first generation.
    #[must_use]
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            generation: Generation::FIRST,
        }
    }
}
