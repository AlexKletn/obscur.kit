A single staged mutation of the retained tree.

A [`TreeTransaction`](super::TreeTransaction) accumulates an ordered list of
these. They are pure data: staging a `TreeChange` never touches the live
tree. The batch is validated and then applied, in order, only at
[`commit`](super::TreeTransaction::commit).

Nodes are referenced through a [`StagedRef`] where a change may target a node
created earlier *in the same transaction*, and through a live [`NodeId`]
where only an already-existing node can be meant.
