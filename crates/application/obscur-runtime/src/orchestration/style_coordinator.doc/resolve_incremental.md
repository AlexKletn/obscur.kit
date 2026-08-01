Re-resolves **only** the nodes `dirty` records, updating `styles` in place
(spec §3.4 phase 5, §4.3).

This is the same walk as [`resolve`](Self::resolve) — the same parent-first
order, the same threaded parent style, the same `rem` base read off the root
element — with the resolve step gated on the dirty set: a node the set does
not record keeps the style it already had instead of recomputing an identical
one. Sharing the walk is what makes the two paths agree by construction rather
than by coincidence.

# What the caller owes

`styles` must be what this same walk produced over this same tree, and
`dirty` must record **every** node whose resolved style can differ — see
[`RestyleScope`](crate::invalidation::RestyleScope), which derives that set
from the selector engine's navigation surface. A node missing from `dirty`
is not checked; it is *assumed* unchanged. Since the marked set is
downward-closed, inheritance needs no separate pass: a node whose parent
changed is in the set with it.
