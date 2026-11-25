Enacts a frame's tree mutation **through** the UI-transaction boundary so the
spec §3.2 rule — "mutations are normalized into validated `UiOps` and
committed atomically between frames" — is realized on the live frame path
rather than only unit-tested.

The frame pipeline used to mutate the retained tree by calling
[`Reconciler::reconcile_root`] directly, bypassing the transaction layer. This
enactor makes the reconcile step go through the validated commit boundary
instead, while keeping reconciliation itself as the underlying mechanism:

1. **Stage** the intent as a [`UiTransaction`] carrying a single
   [`UiOperation::ReplaceRoot`] — the normalized "the root desired output
   changed, reconcile the whole tree" operation.
2. **Validate** the staged batch with a [`TransactionValidator`] *before*
   application; a rejected batch aborts with no mutation (a single
   `ReplaceRoot` is always structurally valid, so this never rejects in
   practice, but the gate is real and observable).
3. **Apply** the reconcile as the commit's effect: `reconcile_root` performs
   the actual retained-tree mutation.
4. **Commit** atomically: when the reconcile changed the tree, advance the
   [`VersionStamp`] and record a [`TransactionCommit`] carrying the applied
   operation stamped with the new version; an idempotent (no-op) reconcile
   produces an empty commit and leaves the version untouched.

Because the effect is exactly the previous `reconcile_root` call, the retained
tree — and therefore every downstream style/layout/scene result — is
**identical** to the pre-boundary behavior. What changes is only that the
mutation now flows through a validated, versioned commit that a caller can
observe (via [`version`](Self::version) and the returned [`TransactionCommit`]).
