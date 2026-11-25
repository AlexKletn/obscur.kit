One normalized structural operation performed while reconciling the desired
tree into the retained tree.

A reconcile pass emits an ordered log of these ops (a
[`ReconciliationPlan`](super::ReconciliationPlan)) describing exactly how the
retained tree was transformed: what was created, removed, moved, or updated
in place. The log is for diagnostics, tests, and — later — observers and the
transaction sub-step that wraps a pass in atomic commit boundaries. Reordered
reuse is expressed as [`ReconciliationOp::Move`] rather than a
remove/insert pair, which is how identity is kept across a reorder.
