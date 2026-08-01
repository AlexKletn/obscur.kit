The runtime-side record of a single mounted component instance (spec §2.1).

This is *plain runtime data*, not the erased component object itself
(holding and dispatching the author's `Component` value is the registry's
job in a later sub-step). An instance records its own [`InstanceId`], the
[`ComponentId`] kind it instantiates, its current [`LifecycleState`], and its
owned [`SubscriptionScope`].

The lifecycle-advancing methods move the state through
`Created → Mounted → (Updating ⇄ Mounted)* → Unmounted`. They are
panic-free: an invalid transition is a no-op that returns `false` rather than
aborting. On [`unmount`](Self::unmount) the subscription scope is cleared,
modelling the spec's "automatic cleanup".
