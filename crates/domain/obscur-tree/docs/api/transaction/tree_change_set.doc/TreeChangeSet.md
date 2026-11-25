An ordered, staged batch of [`TreeChange`]s.

This is pure data: it never touches a tree. It is the payload a
[`TreeTransaction`](super::TreeTransaction) accumulates while it is being
built, and — later, as a planned integration — the shape the reconciler will
emit instead of mutating the tree directly.

Alongside the change list it tracks how many
[`CreateNode`](TreeChange::CreateNode) operations have been staged, so the
transaction can hand out stable [`StagedRef::New`](super::StagedRef::New)
indices in stage order.
