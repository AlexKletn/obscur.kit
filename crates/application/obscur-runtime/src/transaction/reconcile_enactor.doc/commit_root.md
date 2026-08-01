Enacts a root reconcile of `tree` toward `desired` through the atomic
transaction commit boundary and returns the committed record.

The returned [`TransactionCommit`] is non-empty (and carries the advanced
[`version`](Self::version)) exactly when the reconcile changed the tree;
it is empty when the reconcile was an idempotent no-op. Callers derive the
frame's `reconciled` flag from [`TransactionCommit::is_empty`], matching
the previous `!plan.is_empty()` semantics precisely.

The underlying [`ReconciliationPlan`] is returned alongside it, because the
commit records the *transaction* (one normalized `ReplaceRoot`) while the
plan records the *structural deltas* — which nodes were inserted, updated,
moved or removed. A caller that maintains per-node state (component
instances) needs those deltas; a tree walk would have to rediscover what
the plan already states.

# Errors

Returns the [`TransactionError`] from validation if the staged batch is
rejected *before* any mutation. The batch is a single
[`UiOperation::ReplaceRoot`], which is always structurally valid, so this
never returns `Err` in practice — the `Result` keeps the validation gate
honest and total rather than assumed.
