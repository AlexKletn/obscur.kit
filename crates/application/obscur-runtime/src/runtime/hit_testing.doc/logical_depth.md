Returns the logical depth of `node`: the number of logical-parent links
between it and the tree root (root itself is depth `0`).

The walk is bounded by the node count so a malformed (cyclic) parent
chain can never loop forever.
