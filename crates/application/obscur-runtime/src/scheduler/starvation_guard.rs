//! Fairness for background work (spec §3.6 "starvation prevention"):
//! [`StarvationGuard`].

/// A deterministic fairness counter that prevents lower-priority background work
/// from being starved (spec §3.6 "starvation prevention for background work").
///
/// This is **not** time-based: it carries no wall-clock and observes no timers.
/// Instead it counts how many times higher-priority frames have been served
/// while a piece of background/low-priority work waited. Once that count reaches
/// a threshold the guard reports the work as starved, letting the scheduler
/// force a promotion so it finally runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StarvationGuard {
    /// The number of preemptions tolerated before the waiting work is starved.
    threshold: u32,
    /// How many higher-priority frames have been served while work waited.
    waited: u32,
}

impl StarvationGuard {
    /// The default preemption threshold: a small, deterministic bound that lets a
    /// handful of urgent frames through before background work must be promoted.
    pub const DEFAULT_THRESHOLD: u32 = 8;

    /// Creates a guard that reports starvation after `threshold` preemptions.
    ///
    /// A `threshold` of `0` disables the guard: it never reports starvation (see
    /// [`is_starved`](StarvationGuard::is_starved)).
    #[must_use]
    pub const fn new(threshold: u32) -> Self {
        Self {
            threshold,
            waited: 0,
        }
    }

    /// Records that a higher-priority frame was served while the guarded work
    /// waited, bringing it one step closer to starvation.
    ///
    /// The count saturates, so a very long wait cannot overflow.
    pub fn record_preempted(&mut self) {
        self.waited = self.waited.saturating_add(1);
    }

    /// Records that the guarded background/low-priority work was finally served,
    /// resetting the wait count.
    pub fn record_served(&mut self) {
        self.waited = 0;
    }

    /// Returns whether the guarded work is now starved and should be promoted.
    ///
    /// This is `waited >= threshold`, except that a `threshold` of `0` disables
    /// the guard and always returns `false`.
    #[must_use]
    pub const fn is_starved(&self) -> bool {
        self.threshold > 0 && self.waited >= self.threshold
    }

    /// Returns how many preemptions have accumulated since the last service.
    #[must_use]
    pub const fn waited(&self) -> u32 {
        self.waited
    }

    /// Returns the preemption threshold at which work is considered starved.
    #[must_use]
    pub const fn threshold(&self) -> u32 {
        self.threshold
    }
}

impl Default for StarvationGuard {
    /// The default guard uses [`StarvationGuard::DEFAULT_THRESHOLD`].
    fn default() -> Self {
        Self::new(Self::DEFAULT_THRESHOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threshold_is_eight() {
        let guard = StarvationGuard::default();
        assert_eq!(guard.threshold(), StarvationGuard::DEFAULT_THRESHOLD);
        assert_eq!(guard.threshold(), 8);
        assert_eq!(guard.waited(), 0);
    }

    #[test]
    fn below_threshold_is_not_starved() {
        let mut guard = StarvationGuard::new(3);
        guard.record_preempted();
        guard.record_preempted();
        assert_eq!(guard.waited(), 2);
        assert!(!guard.is_starved());
    }

    #[test]
    fn reaching_threshold_is_starved() {
        let mut guard = StarvationGuard::new(3);
        for _ in 0..3 {
            guard.record_preempted();
        }
        assert!(guard.is_starved());
    }

    #[test]
    fn record_served_resets_wait() {
        let mut guard = StarvationGuard::new(2);
        guard.record_preempted();
        guard.record_preempted();
        assert!(guard.is_starved());
        guard.record_served();
        assert_eq!(guard.waited(), 0);
        assert!(!guard.is_starved());
    }

    #[test]
    fn zero_threshold_never_starves() {
        let mut guard = StarvationGuard::new(0);
        for _ in 0..1_000 {
            guard.record_preempted();
        }
        assert!(!guard.is_starved());
    }

    #[test]
    fn preemption_count_saturates() {
        let mut guard = StarvationGuard::new(1);
        guard.waited = u32::MAX;
        guard.record_preempted();
        assert_eq!(guard.waited(), u32::MAX);
    }
}
