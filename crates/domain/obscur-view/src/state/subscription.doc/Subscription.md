One recorded dependency of a component on an observable source (spec §3.1).

A subscription names the [`source`](Self::source) it depends on and the
[`StateGeneration`] that source was at when the dependency was recorded.
Together those are exactly what "dependency tracking invalidates only
consumers of changed values" needs: when a source advances, a subscription to
*that* source whose observed generation is behind identifies a consumer that
must re-run — and one to any other source does not.

Ownership is the cleanup mechanism: the token lives inside its
[`SubscriptionScope`](crate::state::SubscriptionScope) and is released when
that scope drops it (§2.1 "automatic cleanup").
