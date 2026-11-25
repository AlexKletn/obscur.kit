//! Identifier for a scheduled task.

/// An opaque identifier for a spawned task, used to correlate completions and
/// discard stale work (spec §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    /// Wraps a raw task handle value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw task handle value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
