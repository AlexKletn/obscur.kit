A staged [`UiTransaction`] wrapped with **commit/cancel** semantics, for
provisional systems such as drag-and-drop (spec §3.2).

A provisional transaction accumulates normalized operations that represent an
in-flight, not-yet-confirmed interaction. Because staging never touches any
domain, the interaction can be abandoned at any time:

- [`cancel`](Self::cancel) is *free* — it simply drops the staged
  transaction; no domain was ever mutated, so there is nothing to roll back.
- [`commit`](Self::commit) is *atomic* — the staged transaction is validated
  first, and only on success is a [`TransactionCommit`] produced. If
  validation fails the error is returned and, again, nothing is applied.

This is the scoped rollback the spec calls for: provisional commit/cancel
rather than heavyweight universal rollback.

[`UiTransaction`]: crate::transaction::UiTransaction
