Rebuilds the spatial hit-test index from the retained [`LayoutBoxes`].

Every node's absolute border-box `bounds` is inserted with a `z_order`
equal to its **logical depth** — the number of logical-parent links from
the tree root. Deeper (more nested) nodes therefore carry a higher
z-order, so [`SpatialIndex::hits_at`] returns the innermost element first
and a pointer over overlapping ancestor/descendant boxes resolves to the
topmost (innermost) node, matching paint order. Ties at equal depth break
deterministically by [`NodeId`] inside the index.
