Returns a mutable reference to the node for `id`, or `None` if the id is
stale or was never valid.

Note that mutating a node through this reference does not itself advance
the tree version; only structural arena mutations
([`alloc`](Self::alloc), [`remove`](Self::remove),
[`set_root`](Self::set_root)) do.
