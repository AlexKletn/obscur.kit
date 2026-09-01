Dropping the per-node state of a node the reconcile removed.

Four maps outlive the tree they are keyed by, for one reason: every one of them is retained across frames so an unchanged node need not be recomputed, and none of them was ever told what to do about a node that stops existing. The style path cannot notice on its own — `resolve_incremental` walks the *live* tree and only inserts, so a key the walk no longer reaches is never visited again, only paid for.
