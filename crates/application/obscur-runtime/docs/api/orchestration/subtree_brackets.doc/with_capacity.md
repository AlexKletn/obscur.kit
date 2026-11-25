An empty recorder whose three per-node tables are already sized for a tree of
`nodes` nodes.

Every node the paint walk pops calls [`enter`](SubtreeBrackets::enter), which
writes one entry into each of `enter_index`, `parent_of` and `depth_of` — so
their final size is the node count, known before the walk begins. Sizing them
here costs one allocation each and saves each table the handful of rehashes it
would otherwise pay growing from empty, where every rehash re-probes every key
inserted so far.

`nodes` is an upper bound, not a promise: a walk that visits fewer nodes (a
subtree, a tree with unreachable nodes) simply leaves the tail unused. The
bracket vectors — clips, transforms, opacities and the rest — are deliberately
NOT reserved: their length depends on what the stylesheet authored, not on how
many nodes exist, and a document with no brackets at all must not allocate for
them.
