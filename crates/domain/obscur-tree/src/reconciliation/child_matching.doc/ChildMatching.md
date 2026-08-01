The result of matching a parent's existing retained children against a
desired child sequence.

This is a pure decision record produced by [`ChildMatcher`](super::ChildMatcher):
it names, for each desired child in order, whether to reuse an existing node
or create a fresh one ([`matches`](Self::matches)), and which existing
children were left unmatched and must be removed with their subtrees
([`removed`](Self::removed)). It performs no tree mutation; the
[`Reconciler`](super::Reconciler) consumes it to drive the actual arena
updates.

The `matches` slice is always aligned one-to-one with the desired children
it was computed from, in the same order.
