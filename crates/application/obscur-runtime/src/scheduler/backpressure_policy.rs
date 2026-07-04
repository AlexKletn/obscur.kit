//! Admission control for pending frames under load (spec §3.5):
//! [`BackpressurePolicy`] and [`BackpressureDecision`].

/// The decision a [`BackpressurePolicy`] reaches for an incoming frame request
/// (spec §3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackpressureDecision {
    /// Admit the request; there is room in the pending set.
    Admit,
    /// Coalesce the request into an already-pending one for the same reason
    /// (§3.5 "updates are coalesced"); nothing new is queued.
    Coalesce,
    /// Drop a stale queued frame so a fresher one can take its place
    /// (§3.5 "stale intermediate frames may be skipped"). Applies when the
    /// pending set is full and the policy is allowed to skip stale frames.
    DropStale,
    /// Reject the request outright: the pending set is full and stale-skipping
    /// is disabled, so the request cannot be admitted right now.
    Reject,
}

/// The policy for admitting frame requests under load (spec §3.5).
///
/// The scheduler is event-driven and must stay bounded: it caps the number of
/// pending frames and may skip stale intermediate frames rather than let work
/// pile up (§3.5). This value captures those two knobs and is a pure function of
/// them — [`decide`](BackpressurePolicy::decide) never touches wall-clock or
/// randomness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackpressurePolicy {
    /// The maximum number of pending frames, or `None` for unbounded.
    max_pending: Option<usize>,
    /// Whether a stale queued frame may be dropped to admit a fresher one
    /// (§3.5 "stale intermediate frames may be skipped").
    skip_stale: bool,
}

impl BackpressurePolicy {
    /// Creates an unbounded policy that skips stale frames.
    ///
    /// This is also the [`Default`]: no cap on pending frames and stale-skipping
    /// enabled.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            max_pending: None,
            skip_stale: true,
        }
    }

    /// Creates a policy that caps pending frames at `max` and skips stale frames.
    #[must_use]
    pub const fn bounded(max: usize) -> Self {
        Self {
            max_pending: Some(max),
            skip_stale: true,
        }
    }

    /// Returns this policy with stale-skipping set to `skip_stale`.
    #[must_use]
    pub const fn with_skip_stale(mut self, skip_stale: bool) -> Self {
        self.skip_stale = skip_stale;
        self
    }

    /// Returns the pending-frame cap, or `None` if unbounded.
    #[must_use]
    pub const fn max_pending(self) -> Option<usize> {
        self.max_pending
    }

    /// Returns whether this policy skips stale intermediate frames.
    #[must_use]
    pub const fn skip_stale(self) -> bool {
        self.skip_stale
    }

    /// Decides how to admit an incoming request.
    ///
    /// - If a request for the same reason is `already_queued`, the incoming one
    ///   coalesces into it ([`BackpressureDecision::Coalesce`]).
    /// - Otherwise, if the pending set is at or over the cap, the policy either
    ///   drops a stale frame ([`BackpressureDecision::DropStale`]) when
    ///   stale-skipping is enabled, or rejects the request
    ///   ([`BackpressureDecision::Reject`]) when it is not.
    /// - Otherwise the request is admitted ([`BackpressureDecision::Admit`]).
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
    /// The default policy is [`BackpressurePolicy::unbounded`].
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
