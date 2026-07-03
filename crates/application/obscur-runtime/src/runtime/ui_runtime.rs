//! The embeddable UI runtime (skeleton).

use obscur_identity::RuntimeId;

/// An embeddable UI runtime. It orchestrates domains and owns the frame
/// pipeline, but never owns the process or an event loop (spec §1.4, §3.7).
///
/// This skeleton holds identity only; the frame pipeline, scheduler and UI
/// transactions are added in Phase 1.
#[derive(Debug)]
pub struct UiRuntime {
    id: RuntimeId,
}

impl UiRuntime {
    /// Creates a runtime with the given identity.
    #[must_use]
    pub fn new(id: RuntimeId) -> Self {
        Self { id }
    }

    /// Returns this runtime's identity.
    #[must_use]
    pub fn id(&self) -> RuntimeId {
        self.id
    }
}
