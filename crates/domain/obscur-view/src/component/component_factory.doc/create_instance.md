Mints the runtime record for a new instance of this kind.

The returned [`ComponentInstance`] is in
[`LifecycleState::Created`](crate::lifecycle::LifecycleState::Created)
and carries the given [`InstanceId`] and this factory's
[`component_id`](Self::component_id). It is bookkeeping only; the
renderable value comes from [`create_component`](Self::create_component).
