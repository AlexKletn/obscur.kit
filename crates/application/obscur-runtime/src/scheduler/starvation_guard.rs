#![doc = include_str!("starvation_guard.doc/_module.md")]

#[doc = include_str!("starvation_guard.doc/StarvationGuard.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StarvationGuard {
    #[doc = include_str!("starvation_guard.doc/threshold.md")]
    threshold: u32,
    #[doc = include_str!("starvation_guard.doc/waited.md")]
    waited: u32,
}

impl StarvationGuard {
    #[doc = include_str!("starvation_guard.doc/DEFAULT_THRESHOLD.md")]
    pub const DEFAULT_THRESHOLD: u32 = 8;

    #[doc = include_str!("starvation_guard.doc/new.md")]
    #[must_use]
    pub const fn new(threshold: u32) -> Self {
        Self {
            threshold,
            waited: 0,
        }
    }

    #[doc = include_str!("starvation_guard.doc/record_preempted.md")]
    pub fn record_preempted(&mut self) {
        self.waited = self.waited.saturating_add(1);
    }

    #[doc = include_str!("starvation_guard.doc/record_served.md")]
    pub fn record_served(&mut self) {
        self.waited = 0;
    }

    #[doc = include_str!("starvation_guard.doc/is_starved.md")]
    #[must_use]
    pub const fn is_starved(&self) -> bool {
        self.threshold > 0 && self.waited >= self.threshold
    }

    #[doc = include_str!("starvation_guard.doc/waited_2.md")]
    #[must_use]
    pub const fn waited(&self) -> u32 {
        self.waited
    }

    #[doc = include_str!("starvation_guard.doc/threshold_2.md")]
    #[must_use]
    pub const fn threshold(&self) -> u32 {
        self.threshold
    }
}

impl Default for StarvationGuard {
    #[doc = include_str!("starvation_guard.doc/default.md")]
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
