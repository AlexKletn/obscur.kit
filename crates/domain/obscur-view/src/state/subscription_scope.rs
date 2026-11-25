//! `SubscriptionScope`: owns subscriptions and releases them on drop.

use crate::state::state_generation::StateGeneration;
use crate::state::{Subscription, SubscriptionId};
use obscur_identity::SignalId;

#[doc = include_str!("../../docs/api/state/subscription_scope.doc/SubscriptionScope.md")]
#[derive(Debug, Default)]
pub struct SubscriptionScope {
    subscriptions: Vec<Subscription>,
    next_id: u64,
}

impl SubscriptionScope {
    /// Creates an empty scope.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
            next_id: 0,
        }
    }

    #[doc = include_str!("../../docs/api/state/subscription_scope.doc/subscribe.md")]
    pub fn subscribe(&mut self, source: SignalId, observed: StateGeneration) -> SubscriptionId {
        let id = SubscriptionId::from_raw(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.subscriptions
            .push(Subscription::new(id, source, observed));
        id
    }

    /// Whether any subscription in this scope depends on `source` and is behind
    /// its `current` generation — i.e. this scope's owner is a consumer of a
    /// value that changed (spec §3.1).
    #[must_use]
    pub fn is_stale_for(&self, source: SignalId, current: StateGeneration) -> bool {
        self.subscriptions
            .iter()
            .any(|s| s.source() == source && s.is_stale_against(current))
    }

    /// Every source this scope depends on, in the order the dependencies were
    /// recorded.
    pub fn sources(&self) -> impl Iterator<Item = SignalId> + '_ {
        self.subscriptions.iter().map(Subscription::source)
    }

    /// Releases a single subscription early, returning it if it was present.
    pub fn unsubscribe(&mut self, id: SubscriptionId) -> Option<Subscription> {
        let index = self.subscriptions.iter().position(|s| s.id() == id)?;
        Some(self.subscriptions.remove(index))
    }

    /// Returns the subscription with `id`, if it is still active.
    #[must_use]
    pub fn get(&self, id: SubscriptionId) -> Option<&Subscription> {
        self.subscriptions.iter().find(|s| s.id() == id)
    }

    /// Number of active subscriptions in this scope.
    #[must_use]
    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    /// Returns `true` if the scope currently owns no subscriptions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }

    #[doc = include_str!("../../docs/api/state/subscription_scope.doc/clear.md")]
    pub fn clear(&mut self) {
        self.subscriptions.clear();
    }
}

impl Drop for SubscriptionScope {
    fn drop(&mut self) {
        // Deterministic, automatic cleanup: dropping the scope releases every
        // subscription it owns. Explicit here to make the contract visible even
        // though the backing `Vec` would also free the tokens on its own.
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriptionScope;
    use crate::state::StateGeneration;
    use obscur_identity::SignalId;

    #[test]
    fn new_scope_is_empty() {
        let scope = SubscriptionScope::new();
        assert!(scope.is_empty());
        assert_eq!(scope.len(), 0);
    }

    #[test]
    fn subscribe_registers_and_mints_unique_ids() {
        let mut scope = SubscriptionScope::new();
        let a = scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        let b = scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        assert_ne!(a, b);
        assert_eq!(scope.len(), 2);
        assert_eq!(
            scope
                .get(a)
                .map(super::super::subscription::Subscription::id),
            Some(a)
        );
    }

    #[test]
    fn unsubscribe_removes_one() {
        let mut scope = SubscriptionScope::new();
        let a = scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        let _b = scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        let removed = scope.unsubscribe(a);
        assert_eq!(removed.map(|s| s.id()), Some(a));
        assert_eq!(scope.len(), 1);
        assert!(scope.get(a).is_none());
        // Removed id is not reused by the next subscription.
        let c = scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        assert_ne!(c, a);
    }

    #[test]
    fn clear_releases_all() {
        let mut scope = SubscriptionScope::new();
        scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        scope.clear();
        assert!(scope.is_empty());
    }

    #[test]
    fn drop_cleans_up() {
        // Exercised for coverage; real assertion is that this does not leak and
        // clears deterministically.
        let mut scope = SubscriptionScope::new();
        scope.subscribe(SignalId::from_raw(1), StateGeneration::INITIAL);
        drop(scope);
    }
}
