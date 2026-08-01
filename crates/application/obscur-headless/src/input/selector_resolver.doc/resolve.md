Resolves `target` to a live node, or [`None`] when nothing matches.

* [`Node`](SelectorTarget::Node) — returned only if the id is still live,
  so a stale id from a previous frame answers `None` rather than a
  recycled node.
* [`TestId`](SelectorTarget::TestId) — the first node, in tree order,
  carrying a matching `test-id` property.
* [`Coordinate`](SelectorTarget::Coordinate) — always `None`: see the
  type-level note. Use [`point_for`](Self::point_for) to aim a dispatch.
