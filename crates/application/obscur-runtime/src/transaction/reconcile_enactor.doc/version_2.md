Returns the version stamped at the most recent committing frame boundary.

It advances by one every time [`commit_root`](Self::commit_root) applies a
reconcile that actually changed the tree, and is unchanged by a no-op
(idempotent) reconcile, so a caller can observe whether a frame committed a
real tree change.
