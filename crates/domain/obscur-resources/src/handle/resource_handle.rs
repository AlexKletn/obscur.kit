//! A typed, versioned resource handle (skeleton).

use obscur_identity::{Generation, ResourceId};

/// A handle to a managed resource. The handle is stable; the underlying bytes
/// load asynchronously and may be versioned/hot-reloaded (spec §5.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHandle {
    /// Stable resource identity.
    pub id: ResourceId,
    /// Version generation, bumped on hot reload.
    pub generation: Generation,
}

impl ResourceHandle {
    /// Creates a handle at its first generation.
    #[must_use]
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            generation: Generation::FIRST,
        }
    }
}
