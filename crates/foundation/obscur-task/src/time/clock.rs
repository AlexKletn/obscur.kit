//! Clock port.

use super::Instant;

/// A monotonic clock. Adapters provide a real system clock; headless tests
/// provide a deterministic one (spec §3.3).
pub trait Clock {
    /// Returns the current instant on this clock's timeline.
    fn now(&self) -> Instant;
}
