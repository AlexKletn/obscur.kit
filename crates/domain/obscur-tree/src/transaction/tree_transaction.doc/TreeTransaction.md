A validated, atomic batch of tree mutations with provisional-cancel
semantics (spec §3.2).

# Staging, not immediate mutation

The retained tree is a generational arena, so a *removed* node's `NodeId`
cannot be cleanly restored. To get real atomicity *and* real provisional
cancel without rolling back removals, a transaction **stages** its changes
and does not touch the live tree until [`commit`](Self::commit):

- staging methods only append [`TreeChange`]s to an ordered
  [`TreeChangeSet`]; the tree is never borrowed while staging;
- [`cancel`](Self::cancel) (or simply dropping the transaction) discards the
  staged changes — because the tree was never touched, this is the
  provisional-cancel path and it is guaranteed not to mutate anything;
- [`commit`](Self::commit) first [`validate`](Self::validate)s the whole
  batch against the live tree; if anything is invalid it returns a
  [`TxError`] and the tree is left completely untouched (atomicity); only if
  the batch is fully valid are the changes applied in order.

Nodes created earlier in the batch are referenced by later changes through a
[`StagedRef::New`] placeholder, resolved to a concrete [`NodeId`] at commit.

# Layering

This is the *tree-level* primitive. Provisional commit/cancel at the
higher **UI-op** level lives in `obscur-runtime` later and builds on this.
Wiring the reconciler to *emit* a [`TreeChangeSet`] (instead of mutating the
tree directly) is likewise a later integration; for now this transaction is
a self-contained tree-mutation primitive.
