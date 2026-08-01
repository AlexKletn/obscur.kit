The nearest logical ancestor of `node` holding a component whose error
boundary is enabled — the boundary a failure escaping `node` belongs to.

The walk is over *logical* parents: a portal moves where a node paints,
not which component it belongs to (spec §4.14), so the boundary that
answers for it is the one that generated it.
