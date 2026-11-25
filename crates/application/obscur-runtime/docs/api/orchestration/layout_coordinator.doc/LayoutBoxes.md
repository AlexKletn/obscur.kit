The absolute [`LayoutBox`] of every node the coordinator arranged, keyed by
[`NodeId`].

Unlike the per-node boxes produced *inside* a [`LayoutAlgorithm`](obscur_layout::algorithm::LayoutAlgorithm) (which are
parent-relative), the boxes in this map are in **absolute** viewport
coordinates: the coordinator accumulates ancestor offsets as it walks down
the tree (spec §4.2).
