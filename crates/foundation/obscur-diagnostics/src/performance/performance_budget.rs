//! A frame/subsystem time budget.

use std::time::Duration;

/// A time budget for a subsystem. Exceeding a budget produces a diagnostic, not
/// a crash (spec §7.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceBudget {
    /// Maximum allotted duration.
    pub limit: Duration,
}

impl PerformanceBudget {
    /// Creates a budget with the given limit.
    #[must_use]
    pub const fn new(limit: Duration) -> Self {
        Self { limit }
    }

    /// Returns whether `elapsed` exceeds this budget.
    #[must_use]
    pub fn is_exceeded(self, elapsed: Duration) -> bool {
        elapsed > self.limit
    }
}
