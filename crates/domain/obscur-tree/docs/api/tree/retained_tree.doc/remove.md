Removes and returns the node for `id`, vacating its slot.

The slot's generation is advanced and its index pushed onto the free-list
for reuse, so the removed `id` (and any copy of it) becomes stale and will
resolve to `None` thereafter. Returns `None` if the id was already stale
or never valid. This is a structural mutation and advances the tree
[`version`](Self::version).

If the removed node was the current root, the root is cleared.
