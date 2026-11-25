//! ReleaseResult (skeleton stub for the spec-defined entity; not yet implemented).

use obscur_identity::RuntimeId;

use super::LifecycleEvent;

/// Pending lifecycle notifications followed by completion of runtime release.
#[derive(Debug)]
pub struct ReleaseResult {
    events: Vec<LifecycleEvent>,
}

impl ReleaseResult {
    pub(crate) fn complete(runtime: RuntimeId, mut events: Vec<LifecycleEvent>) -> Self {
        events.push(LifecycleEvent::ResourceReleaseComplete(runtime));
        Self { events }
    }

    /// Returns the lifecycle notifications in delivery order.
    #[must_use]
    pub fn into_events(self) -> Vec<LifecycleEvent> {
        self.events
    }
}
