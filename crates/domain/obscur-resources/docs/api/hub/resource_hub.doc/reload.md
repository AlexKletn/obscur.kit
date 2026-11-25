Hot-reloads `handle`, returning the new version to stamp the reload job
with (spec §5.15 development hot reload).

Advances the resource's authoritative version by one and restarts loading
([`ResourceState::Loading`] with indeterminate progress). Any in-flight job
stamped with the prior version is now stale and will be rejected by
[`ResourceHub::complete`] / [`ResourceHub::fail`]. Returns `None` if the
handle is unknown.
