//! `SubscriptionScope`: owns subscriptions and releases them on drop.

use crate::state::state_generation::StateGeneration;
use crate::state::subscription::{Subscription, SubscriptionId};

/// Owns a group of [`Subscription`]s and guarantees they are released together.
///
/// Per spec §2.1 a component's subscriptions are *scoped* and *automatically
/// cleaned up*. Since there is no runtime dispatcher yet, cleanup is modelled
/// honestly through ownership: the scope holds the subscription tokens and
/// releases them when it is dropped (see the [`Drop`] impl), or eagerly via
/// [`SubscriptionScope::clear`]. Nothing outside the scope holds a token, so a
/// scope going out of scope deterministically ends every subscription it owns.
///
/// Ids are minted monotonically and never reused within a scope's lifetime, so
/// a stale [`SubscriptionId`] can never alias a later subscription.
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

    /// Registers a subscription opened while the source was at `observed`,
    /// returning its freshly minted id.
    ///
    /// The scope keeps ownership of the token; the returned id lets the caller
    /// refer to it later (for example to [`unsubscribe`](Self::unsubscribe)).
    pub fn subscribe(&mut self, observed: StateGeneration) -> SubscriptionId {
        let id = SubscriptionId::from_raw(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.subscriptions.push(Subscription::new(id, observed));
        id
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

    /// Releases every subscription this scope owns.
    ///
    /// Called automatically when the scope is dropped; exposed so a component
    /// can also tear its subscriptions down eagerly without dropping the scope.
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

    #[test]
    fn new_scope_is_empty() {
        let scope = SubscriptionScope::new();
        assert!(scope.is_empty());
        assert_eq!(scope.len(), 0);
    }

    #[test]
    fn subscribe_registers_and_mints_unique_ids() {
        let mut scope = SubscriptionScope::new();
        let a = scope.subscribe(StateGeneration::INITIAL);
        let b = scope.subscribe(StateGeneration::INITIAL);
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
        let a = scope.subscribe(StateGeneration::INITIAL);
        let _b = scope.subscribe(StateGeneration::INITIAL);
        let removed = scope.unsubscribe(a);
        assert_eq!(removed.map(|s| s.id()), Some(a));
        assert_eq!(scope.len(), 1);
        assert!(scope.get(a).is_none());
        // Removed id is not reused by the next subscription.
        let c = scope.subscribe(StateGeneration::INITIAL);
        assert_ne!(c, a);
    }

    #[test]
    fn clear_releases_all() {
        let mut scope = SubscriptionScope::new();
        scope.subscribe(StateGeneration::INITIAL);
        scope.subscribe(StateGeneration::INITIAL);
        scope.clear();
        assert!(scope.is_empty());
    }

    #[test]
    fn drop_cleans_up() {
        // Exercised for coverage; real assertion is that this does not leak and
        // clears deterministically.
        let mut scope = SubscriptionScope::new();
        scope.subscribe(StateGeneration::INITIAL);
        drop(scope);
    }
}
