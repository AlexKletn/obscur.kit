Resolves `event`'s command for a `target` by walking its logical-element
ancestor chain (target first, then toward the root) and returning the
first binding for that `event` found — so an event on a child of a bound
element still fires that element's command (event bubbling, spec §2.3).
Returns `None` when no node in the chain binds `event`.

Reuses [`AncestorChain`] for the ancestor walk, which is the same
target-plus-ancestors sequence hover already computes.

# Why the node comes back too

The node whose binding matched **is** §2.3's `current_target`: the element
the handler is attached to, as against `target`, the element the event hit.
This walk has always known it — it is the node `find_map` stopped on — and
used to drop it on the floor, returning the bare symbol. Reporting the pair
is what lets [`PointerDispatch::current_target`](super::PointerDispatch)
be a fact rather than a second guess at a walk the runtime already did.

An associated function rather than a method: the search reads the tree and
no registry, so a caller holding a shared borrow of the runtime can ask it.
