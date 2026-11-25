Compares two contexts for **paint order**, from bottom-most to top-most.

The primary key is `z_index` ascending (a higher `z_index` paints later,
i.e. on top). Ties are broken deterministically by the establishing
node's raw [`NodeId`] so the ordering is a total order and paint order is
always well-defined for a given set of contexts.
