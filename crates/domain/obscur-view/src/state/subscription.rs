#![doc = include_str!("subscription.doc/_module.md")]

use crate::state::StateGeneration;
use obscur_identity::SignalId;

#[doc = include_str!("subscription.doc/SubscriptionId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    #[doc = include_str!("subscription.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("subscription.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("subscription.doc/Subscription.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    id: SubscriptionId,
    source: SignalId,
    observed: StateGeneration,
}

impl Subscription {
    #[doc = include_str!("subscription.doc/new.md")]
    #[must_use]
    pub const fn new(id: SubscriptionId, source: SignalId, observed: StateGeneration) -> Self {
        Self {
            id,
            source,
            observed,
        }
    }

    #[doc = include_str!("subscription.doc/id.md")]
    #[must_use]
    pub const fn id(&self) -> SubscriptionId {
        self.id
    }

    #[doc = include_str!("subscription.doc/source.md")]
    #[must_use]
    pub const fn source(&self) -> SignalId {
        self.source
    }

    #[doc = include_str!("subscription.doc/observed_generation.md")]
    #[must_use]
    pub const fn observed_generation(&self) -> StateGeneration {
        self.observed
    }

    #[doc = include_str!("subscription.doc/is_stale_against.md")]
    #[must_use]
    pub fn is_stale_against(&self, current: StateGeneration) -> bool {
        current.is_newer_than(self.observed)
    }
}

#[cfg(test)]
mod tests {
    use super::{Subscription, SubscriptionId};
    use crate::state::StateGeneration;
    use obscur_identity::SignalId;

    #[test]
    fn accessors_round_trip() {
        let source = SignalId::from_raw(9);
        let sub = Subscription::new(
            SubscriptionId::from_raw(3),
            source,
            StateGeneration::INITIAL,
        );
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
