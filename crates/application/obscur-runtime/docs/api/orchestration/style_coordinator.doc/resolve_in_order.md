Resolves every node in a parent-first `order`, threading each node's
already-computed parent style into the resolver.

Because `order` is parent-first, a node's parent is always present in the
accumulating map by the time the node is resolved, so inheritance flows
down correctly. A node with no cascaded inputs still gets a computed style
from the resolver's defaults and inheritance.

`root_element` is the node `rem` resolves against: the topmost **element**
of the walk, which is what `:root` matches. It is not necessarily
`order[0]` — [`RetainedTree::root`] is a synthetic non-element host node
whose first element child is the document's actual root element.

Because the walk is parent-first, that element is reached before anything
that could refer to it, so its computed `font-size` is in hand in time to
become the `rem` base. It is read back off the resolved style rather than
recomputed, so `rem` cannot drift from the `font-size` the root actually
painted at. Until it is reached (i.e. for the synthetic host above it),
`rem` means the initial `font-size` — the same answer `rem` gets *on* the
root element itself.
