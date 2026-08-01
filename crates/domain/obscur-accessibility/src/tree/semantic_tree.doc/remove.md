Removes and returns the node for `id`, unlinking it from its parent's
child list and dropping its own child list, then bumps the version.

Returns `None` (without bumping the version) if no such node exists. If
the removed node was the [`root`](Self::root), the root is cleared.
