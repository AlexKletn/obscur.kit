//! Why a task was cancelled.

/// The reason a cancellation was requested (spec §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancellationReason {
    /// The owning scope was dropped.
    ScopeEnded,
    /// The work became stale and its result would be discarded.
    Superseded,
    /// The runtime is shutting down.
    Shutdown,
}
