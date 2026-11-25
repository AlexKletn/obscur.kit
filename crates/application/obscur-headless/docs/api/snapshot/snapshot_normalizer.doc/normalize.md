Normalizes `snapshot` into a stable, comparable [`NormalizedPipeline`].

The tree's root and node count are copied through. Every node that
appears in `snapshot`'s computed styles or layout boxes becomes one
[`NormalizedNode`] — reading its computed `opacity` (if present) and its
absolute layout `bounds` (if present) — and the rows are emitted **sorted
by [`NodeId`]**. Because the sort is total and deterministic, two
normalizations of equal snapshots are byte-for-byte equal (spec §7.14).

The remaining §7.4 stages are copied through as stable scalar summaries,
each already order-independent: the scene's fill count and balance, the
paint-command stream length, the accessibility snapshot's node count and
root, the render graph's pass count, and the rendered image (if a backend
attached one).
