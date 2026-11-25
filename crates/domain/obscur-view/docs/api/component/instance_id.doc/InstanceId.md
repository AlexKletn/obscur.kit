Stable, opaque identifier for one mounted component instance.

Distinct from [`ComponentId`](obscur_identity::ComponentId): a `ComponentId`
names a *kind* of component, while an `InstanceId` names a single live
instance of that kind. Backed by an integer handle whose value carries no
meaning outside the runtime that mints it.
