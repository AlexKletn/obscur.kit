The context passed to
[`Component::on_mount`](crate::component::Component::on_mount) when an
instance is first mounted (spec §2.1).

It is a thin wrapper that borrows the underlying [`ComponentContext`]; first
mount carries no phase-specific data beyond the context itself, so this type
simply forwards to it. Access the context through
[`context`](Self::context), or transparently via the [`Deref`]/[`DerefMut`]
impls (e.g. to open a scoped subscription during mount).

# Borrows

`MountContext<'ctx, 'scope>` borrows a `ComponentContext<'scope>` mutably for
`'ctx`. Two lifetimes are required because `ComponentContext` is invariant in
its own borrow lifetime, so the wrapper cannot reuse a single lifetime for
both the outer borrow and the context's inner borrows.
