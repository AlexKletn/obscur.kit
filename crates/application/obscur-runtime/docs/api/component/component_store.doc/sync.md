Brings the store in line with a reconcile that just ran: mounts an
instance for every inserted node whose component kind is registered, and
unmounts every removed node's instance.

Driven by the [`ReconciliationPlan`] rather than by a tree walk, because
the plan already states the exact structural deltas — and driven from
*here* rather than from inside the reconciler, which lives in the domain
tier and can neither see a registry nor accumulate state.

An `Update` op keeps the instance — the node's identity did not change, so
its local state rides along, which is the whole point of keyed identity —
but marks it stale, because the node's authored props changed and the
component's typed props may now decode differently.

A `Move` op is ignored outright: it repositions a node without touching
its content, so the component's output is still correct.
