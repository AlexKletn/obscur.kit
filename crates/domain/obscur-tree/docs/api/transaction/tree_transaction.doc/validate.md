Validates the whole staged batch against `tree` **without mutating it**.

Checks, in stage order:
- every [`StagedRef::Existing`] / `NodeId` target resolves to a live
  node ([`TxError::StaleNode`]);
- every [`StagedRef::New`] reference points to a creation staged
  *earlier* in this transaction ([`TxError::UndefinedNewRef`]);
- each [`TreeChange::SetText`] targets a live text node
  ([`TxError::NotTextNode`]);
- child indices are within range ([`TxError::IndexOutOfRange`]).

# Index-validation rule (projected child counts)

For [`TreeChange::CreateNode`] the parent's *projected* child count is
tracked: it starts at the parent's live child count (for an
[`StagedRef::Existing`] parent) or `0` (for a [`StagedRef::New`] parent),
and grows by one after each valid create under that parent. A valid
insertion index is `0 ..= projected_count`. This precisely models
sequential inserts into the same parent within one batch (e.g. staging
two children into a fresh parent at indices `0` then `1`).

For [`TreeChange::MoveChild`] the index is validated against the *live*
child count of `target`'s current logical parent (`index < count`); a
target that is not currently a child of any live parent accepts any
index. [`TreeChange::RemoveSubtree`] and [`TreeChange::MoveChild`] do not
adjust projected counts — only creates do — which keeps the model simple
and deterministic for the create-then-arrange batches this primitive is
built for. Application additionally clamps every index, so an index the
model does not reject can never cause a panic.

# Errors

Returns a [`TxError`] when a change references a stale or undefined node,
or supplies a child index outside the projected range.
