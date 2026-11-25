Hands a failure that escaped `node`'s disabled boundary to the nearest
enclosing enabled one (spec §1.7), returning that boundary's node and the
fallback UI to commit over its subtree.

Returns [`None`] when no enclosing component contains it: the failure
escapes the component tree entirely, which is what a host asked for by
disabling every boundary above it. It is still reported — the caller has
already done so — but nothing recovers it.
