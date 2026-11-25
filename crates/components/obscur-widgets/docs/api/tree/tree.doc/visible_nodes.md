Returns the indices of the visible nodes: those whose every ancestor is
expanded, in pre-order.

The flat-tree visibility rule walks the nodes in order. When a collapsed
parent at depth `d` is encountered, the following contiguous run of nodes
with depth greater than `d` (its whole subtree) is hidden until a node at
depth `d` or shallower is reached; the collapsed parent itself stays
visible.
