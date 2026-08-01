The ids of the IR nodes whose lowering the data can change: a node with a
bound property or a body interpolation, and every ancestor of one. A node
outside this set lowers identically on every frame, so the bridge's own
result for it is reused verbatim.
