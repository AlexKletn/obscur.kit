//! `Subscription`: an owned token representing one registered subscription.

use crate::state::StateGeneration;
use obscur_identity::SignalId;

use super::SubscriptionId;

#[doc = include_str!("../../docs/api/state/subscription.doc/Subscription.md")]
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

    #[doc = include_str!("../../docs/api/state/subscription.doc/is_stale_against.md")]
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
