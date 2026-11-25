Builds the chain for `target`: the target followed by its element (and
root) ancestors, topmost-first toward the root.

Empty when `target` is `None` (the pointer is over nothing). The walk is
bounded by the node count so a malformed parent chain terminates.
