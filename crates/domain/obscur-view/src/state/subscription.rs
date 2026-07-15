//! `Subscription`: an owned token representing one registered subscription.

use obscur_identity::SignalId;
use crate::state::StateGeneration;

/// Opaque identifier for a subscription within a single
/// [`SubscriptionScope`](crate::state::SubscriptionScope).
///
/// Ids are minted by the owning scope and are unique only within it; the raw
/// value carries no cross-scope meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Wraps a raw handle value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// One recorded dependency of a component on an observable source (spec §3.1).
///
/// A subscription names the [`source`](Self::source) it depends on and the
/// [`StateGeneration`] that source was at when the dependency was recorded.
/// Together those are exactly what "dependency tracking invalidates only
/// consumers of changed values" needs: when a source advances, a subscription to
/// *that* source whose observed generation is behind identifies a consumer that
/// must re-run — and one to any other source does not.
///
/// Ownership is the cleanup mechanism: the token lives inside its
/// [`SubscriptionScope`](crate::state::SubscriptionScope) and is released when
/// that scope drops it (§2.1 "automatic cleanup").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    id: SubscriptionId,
    source: SignalId,
    observed: StateGeneration,
}

impl Subscription {
    /// Creates a subscription token bound to `id`, recording a dependency on
    /// `source` while it was at generation `observed`.
    #[must_use]
    pub const fn new(id: SubscriptionId, source: SignalId, observed: StateGeneration) -> Self {
        Self {
            id,
            source,
            observed,
        }
    }

    /// Returns this subscription's id.
    #[must_use]
    pub const fn id(&self) -> SubscriptionId {
        self.id
    }

    /// The observable source this subscription depends on.
    #[must_use]
    pub const fn source(&self) -> SignalId {
        self.source
    }

    /// Returns the source generation observed when the subscription was opened.
    #[must_use]
    pub const fn observed_generation(&self) -> StateGeneration {
        self.observed
    }

    /// Returns `true` if `current` is newer than the observed generation, i.e.
    /// the subscribed source changed since this subscription last synced.
    ///
    /// `current` must be the generation of [`source`](Self::source); comparing
    /// against a different source's generation is meaningless.
    #[must_use]
    pub fn is_stale_against(&self, current: StateGeneration) -> bool {
        current.is_newer_than(self.observed)
    }
}

#[cfg(test)]
mod tests {
    use obscur_identity::SignalId;
    use super::{Subscription, SubscriptionId};
    use crate::state::StateGeneration;

    #[test]
    fn accessors_round_trip() {
        let source = SignalId::from_raw(9);
        let sub = Subscription::new(SubscriptionId::from_raw(3), source, StateGeneration::INITIAL);
        assert_eq!(sub.id().raw(), 3);
        assert_eq!(sub.source(), source);
        assert_eq!(sub.observed_generation(), StateGeneration::INITIAL);
    }

    #[test]
    fn staleness_follows_generation() {
        let seen = StateGeneration::from_raw(4);
        let sub = Subscription::new(SubscriptionId::from_raw(1), SignalId::from_raw(1), seen);
        assert!(sub.is_stale_against(seen.next()));
        assert!(!sub.is_stale_against(seen));
    }
}
