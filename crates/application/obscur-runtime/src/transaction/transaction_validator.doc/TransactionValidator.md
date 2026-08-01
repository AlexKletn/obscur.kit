A stateless validator for [`UiTransaction`]s.

Per spec §3.2, normal UI commits are *validated before application*. This
validator performs the **structural** checks that require only the batch
itself — it never mutates anything and holds no state:

- at most one [`UiOperation::ReplaceRoot`] per batch, otherwise
  [`TransactionError::ConflictingRootReplacement`] (a frame can reconcile at
  most one new root);
- a node is not targeted twice by the same kind of operation within the
  batch, otherwise [`TransactionError::DuplicateNodeOperation`].

**Liveness** validation — checking that targeted nodes actually exist in the
tree — is deliberately *not* done here; it is applied later in the frame
pipeline, where the live tree is available. Keeping this validator
tree-agnostic is what lets it stay stateless and reusable for provisional
(commit/cancel) systems.

[`UiTransaction`]: crate::transaction::UiTransaction
