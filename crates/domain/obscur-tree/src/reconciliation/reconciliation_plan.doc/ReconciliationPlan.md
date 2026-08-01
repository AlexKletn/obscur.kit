An ordered record of the [`ReconciliationOp`]s produced by a reconcile pass.

The plan is the op log a [`Reconciler`](super::Reconciler) emits while
transforming the retained tree from a desired tree: a flat, ordered sequence
of the inserts, removes, moves, and in-place updates it performed. It exists
for diagnostics, tests, and — later — observers.

The transaction sub-step will wrap a reconcile pass in atomic `UiOps` commit
boundaries; here the plan is simply the produced op log, with no transactional
guarantees of its own.
