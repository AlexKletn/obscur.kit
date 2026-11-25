Ensures a root node exists and reconciles its children against `desired`.

If the tree already has a live root it is reused; otherwise a
[`NodeKind::Root`] node is allocated and designated as the root (recorded
as an [`ReconciliationOp::Insert`] with the root as its own parent). The
desired [`View`]'s top-level fragment then reconciles into that root's
children.
