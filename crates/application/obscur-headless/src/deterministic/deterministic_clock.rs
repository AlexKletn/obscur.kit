#![doc = include_str!("deterministic_clock.doc/_module.md")]

use std::cell::Cell;

use obscur_task::{Clock, Instant};

#[doc = include_str!("deterministic_clock.doc/DeterministicClock.md")]
#[derive(Debug, Default)]
pub struct DeterministicClock {
    nanos: Cell<u64>,
}

impl DeterministicClock {
    #[doc = include_str!("deterministic_clock.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("deterministic_clock.doc/advance.md")]
    pub fn advance(&self, nanos: u64) {
        self.nanos.set(self.nanos.get().saturating_add(nanos));
    }
}

impl Clock for DeterministicClock {
    fn now(&self) -> Instant {
        Instant::from_nanos(self.nanos.get())
    }
}
