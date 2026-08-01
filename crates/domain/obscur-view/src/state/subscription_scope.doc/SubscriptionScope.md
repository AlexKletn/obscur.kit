Owns a group of [`Subscription`]s and guarantees they are released together.

Per spec §2.1 a component's subscriptions are *scoped* and *automatically
cleaned up*. Since there is no runtime dispatcher yet, cleanup is modelled
honestly through ownership: the scope holds the subscription tokens and
releases them when it is dropped (see the [`Drop`] impl), or eagerly via
[`SubscriptionScope::clear`]. Nothing outside the scope holds a token, so a
scope going out of scope deterministically ends every subscription it owns.

Ids are minted monotonically and never reused within a scope's lifetime, so
a stale [`SubscriptionId`] can never alias a later subscription.
