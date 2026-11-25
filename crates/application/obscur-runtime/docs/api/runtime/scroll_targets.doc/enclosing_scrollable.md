The nearest **user-scrollable** ancestor container of `node` (the node
itself first, then toward the root), or [`None`] when nothing above it
scrolls.

Innermost first: [`AncestorChain`] is node-first, so the first ancestor that
is a scrollable container is the one that owns this node's scrolling.
