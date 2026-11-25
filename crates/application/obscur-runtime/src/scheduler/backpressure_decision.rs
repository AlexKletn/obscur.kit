/// The decision a [`BackpressurePolicy`](super::BackpressurePolicy) reaches for an incoming frame request
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
