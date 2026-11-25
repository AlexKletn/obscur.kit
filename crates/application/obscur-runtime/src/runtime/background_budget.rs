//! Host-configurable limits for servicing background work.

/// Maximum number of background queue entries to service per poll, scoped to
/// one runtime. The default is one in either scheduling mode. Cancelled and
/// expired entries consume a slot when discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundBudget {
    /// Limit when no frame is due. Zero disables idle servicing.
    pub idle_units: usize,
    /// Limit before a due frame when the fairness threshold has been reached.
    /// Zero disables service before frames.
    pub fairness_units: usize,
}

impl Default for BackgroundBudget {
    fn default() -> Self {
        Self {
            idle_units: 1,
            fairness_units: 1,
        }
    }
}
