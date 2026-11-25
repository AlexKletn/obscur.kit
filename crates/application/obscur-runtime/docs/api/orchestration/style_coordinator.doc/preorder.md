Computes a parent-first (pre-order) traversal from `root`, pairing each
visited node with its parent id (the root's parent is [`None`]).

`children_of` yields the child ids of a node. The traversal is cycle-safe:
a visited set guarantees each node is emitted at most once, so cyclic or
diamond child links terminate rather than loop. Children are emitted in
their natural left-to-right order.
