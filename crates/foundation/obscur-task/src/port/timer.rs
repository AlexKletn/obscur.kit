//! Timer port.

use std::time::Duration;

/// Schedules a wake-up after a delay. Adapters back this with the platform
/// timer facility; headless tests use a deterministic queue (spec §3.5).
pub trait Timer {
    /// Requests a frame/wake-up after `delay`.
    fn schedule(&self, delay: Duration);
}
