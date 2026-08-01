The **arrange** phase input for a node (spec §4.2): the final region
(a rectangle in parent-relative space) the parent assigns to the node.

Arrange positions a node (and recursively its children) into its assigned
region, producing a [`LayoutBox`](crate::model::LayoutBox). It runs after
the measure phase ([`MeasurePass`](super::MeasurePass)), which decides sizes
without positioning.
