//! Headless host wiring a runtime to a deterministic clock (skeleton).

use obscur_identity::RuntimeId;
use obscur_runtime::UiRuntime;

use crate::deterministic::DeterministicClock;

/// A windowless, GPU-less host that drives a [`UiRuntime`] with a deterministic
/// clock, for snapshot and behavioural tests (spec §7.4).
#[derive(Debug)]
pub struct HeadlessHost {
    runtime: UiRuntime,
    clock: DeterministicClock,
}

impl HeadlessHost {
    /// Creates a headless host owning a runtime with the given identity.
    #[must_use]
    pub fn new(runtime_id: RuntimeId) -> Self {
        Self {
            runtime: UiRuntime::new(runtime_id),
            clock: DeterministicClock::new(),
        }
    }

    /// Returns the hosted runtime.
    #[must_use]
    pub fn runtime(&self) -> &UiRuntime {
        &self.runtime
    }

    /// Returns the deterministic clock driving this host.
    #[must_use]
    pub fn clock(&self) -> &DeterministicClock {
        &self.clock
    }
}
