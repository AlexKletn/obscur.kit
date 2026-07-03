//! `Subscription`: an owned token representing one registered subscription.

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

/// An owned token standing in for one active subscription.
///
/// There is no reactive dispatcher yet (that lives in `obscur-runtime`), so a
/// subscription is modelled honestly as a small owned handle rather than a live
/// callback registration: it records the [`StateGeneration`] of the source it
/// was opened against. Ownership is the cleanup mechanism — the token lives
/// inside its [`SubscriptionScope`](crate::state::SubscriptionScope) and is
/// released when that scope drops it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    id: SubscriptionId,
    observed: StateGeneration,
}

impl Subscription {
    /// Creates a subscription token bound to `id`, opened while the source was
    /// at generation `observed`.
    #[must_use]
    pub const fn new(id: SubscriptionId, observed: StateGeneration) -> Self {
        Self { id, observed }
    }

    /// Returns this subscription's id.
    #[must_use]
    pub const fn id(&self) -> SubscriptionId {
        self.id
    }

    /// Returns the source generation observed when the subscription was opened.
    #[must_use]
    pub const fn observed_generation(&self) -> StateGeneration {
        self.observed
    }

    /// Returns `true` if `current` is newer than the observed generation, i.e.
    /// the subscribed source changed since this subscription last synced.
    #[must_use]
    pub fn is_stale_against(&self, current: StateGeneration) -> bool {
        current.is_newer_than(self.observed)
    }
}

#[cfg(test)]
mod tests {
    use super::{Subscription, SubscriptionId};
    use crate::state::StateGeneration;

    #[test]
    fn accessors_round_trip() {
        let sub = Subscription::new(SubscriptionId::from_raw(3), StateGeneration::INITIAL);
        assert_eq!(sub.id().raw(), 3);
        assert_eq!(sub.observed_generation(), StateGeneration::INITIAL);
    }

    #[test]
    fn staleness_follows_generation() {
        let seen = StateGeneration::from_raw(4);
        let sub = Subscription::new(SubscriptionId::from_raw(1), seen);
        assert!(sub.is_stale_against(seen.next()));
        assert!(!sub.is_stale_against(seen));
    }
}
