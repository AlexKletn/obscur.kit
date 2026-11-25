Resolves `target` to a live node, or [`None`] when there is no unique match.

* [`Node`](SelectorTarget::Node) — returned only if the id is still live,
  so a stale id from a previous frame answers `None` rather than a
  recycled node.
* [`TestId`](SelectorTarget::TestId) and [`AutomationId`](SelectorTarget::AutomationId)
  resolve a unique node carrying the corresponding property.
* [`Coordinate`](SelectorTarget::Coordinate) — always `None`: see the
  type-level note. Use [`point_for`](Self::point_for) to aim a dispatch.

A semantic role and optional accessible name resolve only when exactly one
visible node matches the runtime's accessibility projection.
