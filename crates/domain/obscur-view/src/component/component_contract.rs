//! Typed component contract (skeleton).

use obscur_identity::ComponentId;

/// The typed, exported contract of a component (spec §2.13). This skeleton
/// carries only identity; props/events/slots are added in Phase 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContract {
    /// Stable component identity.
    pub id: ComponentId,
}

impl ComponentContract {
    /// Creates a contract for the given component id.
    #[must_use]
    pub fn new(id: ComponentId) -> Self {
        Self { id }
    }
}
