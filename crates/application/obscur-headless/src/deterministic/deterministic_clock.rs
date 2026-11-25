//! A fully deterministic, manually-advanced clock (spec §3.3, §7.4).

use std::cell::Cell;

use obscur_task::{Clock, Instant};

/// A clock whose time only moves when [`advance`](Self::advance) is called,
/// giving headless tests reproducible timing.
#[derive(Debug, Default)]
pub struct DeterministicClock {
    nanos: Cell<u64>,
}

impl DeterministicClock {
    /// Creates a clock at time zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances virtual time by `nanos` nanoseconds (saturating).
    pub fn advance(&self, nanos: u64) {
        self.nanos.set(self.nanos.get().saturating_add(nanos));
    }
}

impl Clock for DeterministicClock {
    fn now(&self) -> Instant {
        Instant::from_nanos(self.nanos.get())
    }
}
