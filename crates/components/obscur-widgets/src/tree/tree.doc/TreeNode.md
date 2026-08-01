A single node in a flat [`Tree`] model.

Hierarchy is expressed by `depth` (the root nodes are depth `0`, their
children depth `1`, and so on) rather than by nested ownership: a `Tree`
stores its nodes in a flat, pre-order [`Vec`], and a node's children are the
contiguous run of following nodes with a greater depth. `has_children`
records whether the node is an expandable parent.
