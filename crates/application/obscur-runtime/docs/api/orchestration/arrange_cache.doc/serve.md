Copies `node`'s retained subtree boxes into `boxes`, reporting whether it
could.

# Why this writes exactly what a full `arrange` would

The previous pass, from this same `node` under this same key over a subtree
nothing has changed, wrote some set `S` of boxes. `S ⊆ tree-subtree(node)`,
because that is where the recursion goes. And every node in
`tree-subtree(node) \ S` has **no** retained entry, because the only call
that ever writes a node's box is the one that reaches it through this same
chain of child links. So "every retained entry in `tree-subtree(node)`" *is*
`S` — which is why this copies what it finds rather than what the tree says
should be there. That distinction is load-bearing: `arrange` writes no box
for a node whose `display` has no algorithm, and zips children against the
algorithm's returned boxes, so `S` is a subset the tree shape alone does not
predict.

Returns `false` — having written nothing — when `node` itself has no
retained box, which is the case a full arrange must handle anyway.
