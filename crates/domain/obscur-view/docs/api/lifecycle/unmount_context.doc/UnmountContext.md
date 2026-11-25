The context passed to
[`Component::on_unmount`](crate::component::Component::on_unmount) as an
instance is torn down (spec §2.1).

It borrows the underlying [`ComponentContext`] so the hook can perform any
last read of its subscription scope or phase. It carries no extra data: the
actual teardown is *automatic*. Dropping the component's
[`SubscriptionScope`](crate::state::SubscriptionScope) releases every scoped
subscription, and dropping its [`LocalState`](crate::state::LocalState)
fields releases its local view state. This hook exists only for side-effects
the author wants to run *before* that automatic cleanup happens; it is not
required for correctness.

# Borrows

`UnmountContext<'ctx, 'scope>` borrows a `ComponentContext<'scope>` mutably
for `'ctx`, using two lifetimes for the same invariance reason as
[`MountContext`](crate::lifecycle::MountContext).
