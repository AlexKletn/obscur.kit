An accumulator of pending node restyles keyed by [`NodeId`] (§4.3).

Each node maps to the typed [`StyleInvalidationReason`] that first put it in
the set. When a node is invalidated more than once the first reason is kept
(documented precedence: earliest wins); merging never panics and never loses a
node.

# Who reads it

The runtime's style phase, on the interaction path: `RestyleScope` fills one of
these from the nodes whose `:hover`/`:focus` state changed,
`StyleMatchingCoordinator::rematch_into` iterates it to re-match exactly those
elements, and `StyleCoordinator::resolve_incremental` reads
[`is_invalidated`](Self::is_invalidated) to decide, per node, whether to
recompute or keep the retained style. A frame whose style inputs moved for any
*other* reason — a new stylesheet, a resize, a tree edit — builds no set and
restyles in full, so this is consulted on one path, not all of them.

[`take`](Self::take)/[`drain`](Self::drain) are **not** on that path: the set is
read, not consumed, because the same set gates both the re-match and the
re-resolve. They have no production caller.

The tree walk — finding the children of a changed parent, or the node whose
state toggled — is the runtime's job; this type only records and merges the
invalidations that walk produces.
