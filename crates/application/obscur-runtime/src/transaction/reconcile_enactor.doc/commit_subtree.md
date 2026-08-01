Enacts a **subtree** reconcile — one mounted component's freshly rendered
output into its own children — through the same commit boundary, and
returns the committed record plus the structural deltas.

This is the [`UiOperation::Rerender`] counterpart of
[`commit_root`](Self::commit_root): where a root replacement restates the
whole desired tree, a rerender restates exactly one component's output.
The tree below `node` is the only thing that can change, so a component
whose local state advanced costs its own subtree rather than the document.

Version semantics match `commit_root` exactly: an idempotent rerender —
a component that re-rendered to the same output — commits empty and leaves
the stamp untouched, so a caller can tell a real change from a no-op.

# Errors

Returns the [`TransactionError`] from validation if the staged batch is
rejected *before* any mutation. A single [`UiOperation::Rerender`] is
always structurally valid, so this does not return `Err` in practice; the
`Result` keeps the validation gate total rather than assumed.
