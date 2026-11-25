Unpacks a [`NodeId`] into its `(index, generation)` pair.

Inverse of [`pack_node_id`]. The masked/shifted halves always fit in a `u32`
by construction, so the conversions never fail; the `unwrap_or(0)` fallbacks
exist only to keep the function total and panic-free.
