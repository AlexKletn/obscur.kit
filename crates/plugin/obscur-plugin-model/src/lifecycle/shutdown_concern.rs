//! Advisory plugin response to a host shutdown request.

use std::time::Duration;

/// Describes pending work and the time requested to finish it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShutdownConcern {
    /// Plugin-supplied description for the host to present or log.
    pub message: Box<str>,
    /// Requested time for pending work. The host can grant less time.
    pub requested_time: Duration,
}
