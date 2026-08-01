The z-order-aware hit-test index and the stacking-order resolution over it
(spec §4.7).

It owns the [`SpatialIndex`] and nothing else: the tree, the computed styles
and the retained boxes are handed in, because they belong to the frame
pipeline and a hit test is a *reader* of what the last frame produced.

# Why the index and the resolver are one entity

They are two halves of one stacking order and they have to agree.
[`rebuild`](Self::rebuild) records each box under a `z_order` equal to the
node's **logical depth**, so [`SpatialIndex::hits_at`] yields the innermost
candidate first; [`resolve`](Self::resolve) then re-derives that same depth as
the middle term of its `(z-index, depth, pre-order)` sort key. Built by one
rule and ranked by another, a pointer over overlapping boxes lands on a node
that is not the one on top — and it lands there silently, because both halves
individually look right.

The second pairing is a staleness one: the index is a projection of the
[`LayoutBoxes`], so a frame that re-arranges boxes without rebuilding resolves
the *previous* frame's geometry, and every click misses by exactly however far
the layout moved.
