//! Notifications delivered to the host.

use obscur_identity::{RuntimeId, WindowId};

use super::{ShutdownConcern, ShutdownRequest};

/// A runtime lifecycle transition observed by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// The final tracked window closed. Carries the ID of that window.
    LastWindowClosed(WindowId),
    /// A request to shut down that the host can handle according to its policy.
    ShutdownRequested(ShutdownRequest),
    /// The runtime and its owned handles have been dropped.
    ResourceReleaseComplete(RuntimeId),
    /// A critical failure reported by the host or runner.
    CriticalStop(ShutdownConcern),
}
