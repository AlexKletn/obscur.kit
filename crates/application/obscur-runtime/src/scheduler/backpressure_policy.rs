#![doc = include_str!("backpressure_policy.doc/_module.md")]

#[doc = include_str!("backpressure_policy.doc/BackpressureDecision.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpressureDecision {
    #[doc = include_str!("backpressure_policy.doc/Admit.md")]
    Admit,
    #[doc = include_str!("backpressure_policy.doc/Coalesce.md")]
    Coalesce,
    #[doc = include_str!("backpressure_policy.doc/DropStale.md")]
    DropStale,
    #[doc = include_str!("backpressure_policy.doc/Reject.md")]
    Reject,
}

#[doc = include_str!("backpressure_policy.doc/BackpressurePolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackpressurePolicy {
    #[doc = include_str!("backpressure_policy.doc/max_pending.md")]
    max_pending: Option<usize>,
    #[doc = include_str!("backpressure_policy.doc/skip_stale.md")]
    skip_stale: bool,
}

impl BackpressurePolicy {
    #[doc = include_str!("backpressure_policy.doc/unbounded.md")]
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            max_pending: None,
            skip_stale: true,
        }
    }

    #[doc = include_str!("backpressure_policy.doc/bounded.md")]
    #[must_use]
    pub const fn bounded(max: usize) -> Self {
        Self {
            max_pending: Some(max),
            skip_stale: true,
        }
    }

    #[doc = include_str!("backpressure_policy.doc/with_skip_stale.md")]
    #[must_use]
    pub const fn with_skip_stale(mut self, skip_stale: bool) -> Self {
        self.skip_stale = skip_stale;
        self
    }

    #[doc = include_str!("backpressure_policy.doc/max_pending_2.md")]
    #[must_use]
    pub const fn max_pending(self) -> Option<usize> {
        self.max_pending
    }

    #[doc = include_str!("backpressure_policy.doc/skip_stale_2.md")]
    #[must_use]
    pub const fn skip_stale(self) -> bool {
        self.skip_stale
    }

    #[doc = include_str!("backpressure_policy.doc/decide.md")]
    #[must_use]
    pub const fn decide(&self, already_queued: bool, queue_len: usize) -> BackpressureDecision {
        if already_queued {
            return BackpressureDecision::Coalesce;
        }
        if let Some(max) = self.max_pending
            && queue_len >= max
        {
            return if self.skip_stale {
                BackpressureDecision::DropStale
            } else {
                BackpressureDecision::Reject
            };
        }
        BackpressureDecision::Admit
    }
}

impl Default for BackpressurePolicy {
    #[doc = include_str!("backpressure_policy.doc/default.md")]
    fn default() -> Self {
        Self::unbounded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbounded_admits_unless_already_queued() {
        let policy = BackpressurePolicy::unbounded();
        assert_eq!(policy.decide(false, 0), BackpressureDecision::Admit);
        assert_eq!(policy.decide(false, 1_000), BackpressureDecision::Admit);
        assert_eq!(policy.decide(true, 0), BackpressureDecision::Coalesce);
    }

    #[test]
    fn default_matches_unbounded() {
        assert_eq!(
            BackpressurePolicy::default(),
            BackpressurePolicy::unbounded()
        );
    }

    #[test]
    fn bounded_below_capacity_admits() {
        let policy = BackpressurePolicy::bounded(4);
        assert_eq!(policy.decide(false, 0), BackpressureDecision::Admit);
        assert_eq!(policy.decide(false, 3), BackpressureDecision::Admit);
    }

    #[test]
    fn bounded_at_capacity_drops_stale_when_enabled() {
        let policy = BackpressurePolicy::bounded(4);
        assert_eq!(policy.decide(false, 4), BackpressureDecision::DropStale);
        assert_eq!(policy.decide(false, 5), BackpressureDecision::DropStale);
    }

    #[test]
    fn bounded_at_capacity_rejects_without_stale_skipping() {
        let policy = BackpressurePolicy::bounded(4).with_skip_stale(false);
        assert_eq!(policy.decide(false, 4), BackpressureDecision::Reject);
    }

    #[test]
    fn already_queued_coalesces_regardless_of_capacity() {
        let policy = BackpressurePolicy::bounded(1).with_skip_stale(false);
        assert_eq!(policy.decide(true, 100), BackpressureDecision::Coalesce);
    }
}
