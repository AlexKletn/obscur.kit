A reference to a node *within a staged transaction*.

While a [`TreeTransaction`](super::TreeTransaction) is being built its
changes have not touched the live tree yet, so a node that the transaction
itself will create does not have a [`NodeId`] until commit time. A
`StagedRef` bridges that gap: it is either an [`Existing`](Self::Existing)
live node, or a [`New`](Self::New) placeholder naming the *i*-th node staged
for creation in this transaction (0-based, in stage order).

At [`commit`](super::TreeTransaction::commit) each `New(i)` is resolved to
the concrete `NodeId` returned by the *i*-th allocation, so a node created
earlier in the batch can be referenced (e.g. as a parent) by later changes.
