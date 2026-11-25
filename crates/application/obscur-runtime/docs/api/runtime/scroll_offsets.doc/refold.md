Recomputes the scroll-chain-adjusted view of the retained layout boxes that
[`projection`](Self::projection) exposes.

The runtime's own boxes stay in content space — paint/compose and hit-test
read them directly and apply scroll themselves. The projection instead
reports each box translated up by the summed offset of every overflowing
scroll container on its ancestor chain. When nothing overflows,
[`folded_active`](Self::folded_active) is cleared and the accessor returns
the content-space boxes with no clone (the common case).
