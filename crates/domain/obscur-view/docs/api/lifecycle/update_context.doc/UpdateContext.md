The context passed to
[`Component::on_update`](crate::component::Component::on_update) when an
instance's inputs may have changed (spec §2.1, §3.1).

Besides borrowing the underlying [`ComponentContext`], it carries the
previous and next input [`StateGeneration`]s so the hook can early-out when
nothing it depends on actually changed. This ties into the spec's
"invalidate only changed consumers": [`changed`](Self::changed) is `true`
only when the next generation is strictly newer than the previous one.

# Borrows

`UpdateContext<'ctx, 'scope>` borrows a `ComponentContext<'scope>` mutably
for `'ctx` (two lifetimes for the same invariance reason as
[`MountContext`](crate::lifecycle::MountContext)) and additionally owns two
`StateGeneration` watermarks by value.
