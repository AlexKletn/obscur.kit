Rebuilds the spatial hit-test index from the retained [`LayoutBoxes`].

Every node is inserted with a `z_order` equal to its **logical depth** —
the number of logical-parent links from the tree root. Deeper (more
nested) nodes therefore carry a higher z-order, so
[`SpatialIndex::hits_at`] returns the innermost element first and a
pointer over overlapping ancestor/descendant boxes resolves to the
topmost (innermost) node, matching paint order. Ties at equal depth break
deterministically by [`NodeId`] inside the index.

The inserted rect is the node's absolute border box **as it appears on
screen**: for an untransformed node that is `bounds` verbatim, and for a
node under a CSS `transform` it is the axis-aligned bounding box of the
transformed corners. The index stays a broad phase either way — it may
over-report a rotated node in the empty corners of that box, which
[`resolve`](Self::resolve) then rejects by mapping the point back through
the inverse. Inserting the raw `bounds` instead would let the broad phase
*under*-report, and a miss there is unrecoverable.

A node whose accumulated transform is singular is not indexed at all: it
collapses to zero area, paints nothing, and so is hit by nothing.
