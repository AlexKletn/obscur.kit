Validates the batch, then applies it in order, returning a
[`TreeCommit`] recording the concrete effects.

If [`validate`](Self::validate) fails, the error is returned and the tree
is left **completely untouched** (atomicity). Otherwise every change is
applied in stage order:
- [`StagedRef::New`] refs are resolved to the id returned by the matching
  allocation;
- a create allocates the node, sets its logical *and* visual parent to
  the resolved parent, and inserts it into the parent's children at the
  (clamped) index;
- a subtree removal detaches the target from its parent and removes the
  subtree leaves-first;
- a move repositions the child within its current parent;
- a set-text replaces the node's kind with the new text;
- a set-root designates the resolved node as root.

Because application runs only after successful validation, and validated
tree operations do not fail, the apply phase is atomic in practice.

# Errors

Returns a [`TxError`] if validation of the staged change set fails; in
that case the tree is left unchanged.
