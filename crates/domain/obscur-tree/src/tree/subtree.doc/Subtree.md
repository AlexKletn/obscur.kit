A lightweight handle identifying a node and its transitive descendants.

A `Subtree` is just a root [`NodeId`]; it owns no node data. All queries are
resolved against a borrowed [`RetainedTree`] by following the by-`NodeId`
[`children`](crate::node::RetainedNode::children) links (spec §2.2). Traversal
is deterministic (pre-order, depth-first) and defensive against malformed or
cyclic links: every node is visited at most once, so a query can never panic
or loop forever.
