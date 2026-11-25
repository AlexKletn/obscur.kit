Records every IR node whose lowering the data can change — one with a bound
property or a body interpolation, and every ancestor of one — returning whether
`node`'s own subtree is such a node.

An interpolation counts whatever its expression is (a literal `{7}` included),
so a body `{…}` renders its value consistently rather than depending on whether
a sibling happened to be bound.

A bound `key={…}` deliberately does **not** count: a key is reconciliation
identity, and this route materialises a loop statically, so there is nothing
per-item to key and substituting one would change node identity between frames
for no gain.
