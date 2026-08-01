Returns the version stamped at the most recent frame that committed a real
tree change through the transaction boundary (spec §3.2).

It advances by one each time a frame's reconcile actually mutates the
retained tree and is unchanged by an idempotent (no-op) reconcile or a
frame that skips the reconcile phase — so a host can observe that the live
tree mutation went through a validated, atomic [`UiTransaction`] commit
rather than a bare `reconcile_root`.

[`UiTransaction`]: crate::transaction::UiTransaction
