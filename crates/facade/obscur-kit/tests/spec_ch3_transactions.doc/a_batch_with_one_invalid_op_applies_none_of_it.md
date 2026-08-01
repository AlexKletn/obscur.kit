§3.2 — "Mutations are normalized into validated `UiOps` and committed
atomically between frames."

A batch whose *second* op is invalid must be rejected as a whole: the first
op's effect must not appear, and no frame is requested.

WEAKENED 2026-07-19: the batch's invalid op used to be an
`InvalidateLayout` aimed at a NodeId that is not in the tree, and the case
asserted that a dead target rejects the batch. §3.2 says "Mutations are
normalized into validated `UiOps` and committed atomically between frames."
and "- normal UI commits are validated before application;" — it says commits
ARE validated, and nowhere enumerates what validation checks. Target liveness
is not among the things the text requires, and `TransactionValidator` is by
construction stateless and tree-less, so it could not check it. The invalid
op is now one the spec's own vocabulary does cover — a normalized batch that
contradicts itself — so the case still tests the property it is named for:
atomicity. Candidate spec amendment: state what §3.2 validation is obliged to
check, and in particular whether an op targeting a node that no longer exists
is rejected, dropped, or applied as a no-op — three defensible answers with
very different consequences for an app that mutates across a frame boundary.
