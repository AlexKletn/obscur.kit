A single invalidation record: a runtime-level [`InvalidationReason`] together
with the [`NodeId`] it applies to.

The node is `None` for a global change (such as
[`InvalidationReason::EnvironmentChanged`]) that is not scoped to a single
node; in that case the frame pipeline invalidates the whole tree.
