The incremental spatial index used for hit testing (spec §4.7).

The index stores each node's resolved visual [`bounds`](Entry::bounds) and
[`z_order`](Entry::z_order). It is maintained *incrementally* — nodes are
inserted, removed, and re-bounded individually rather than rebuilt — and is
backed by a [`BTreeMap`] keyed by [`NodeId`] for deterministic iteration.

Queries return [`HitTest`] records ordered topmost-first (descending
z-order, ties broken by `NodeId`).
