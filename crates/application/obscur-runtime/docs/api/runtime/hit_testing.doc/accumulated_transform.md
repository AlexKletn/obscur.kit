The logical depth and composed CSS `transform` of `node` and its ancestors.
The transform is `None` when nothing along that chain transforms.

Painting brackets a transformed node's whole subtree, so a child of a
rotated card is rotated too even though it declares no transform of its
own; hit testing has to see the same chain or the two disagree. The
result maps the node's untransformed border-box space to screen space —
`ancestors · own`, since a point leaves the node's own space first.

The memo carries depth and transform already computed this rebuild. The walk climbs
to the nearest memoized ancestor (or the root), then folds back down
recording every step, so a rebuild composes each node's own transform
exactly once no matter how deep the tree. The ancestor walk is capped at
the node count so a malformed cyclic parent chain terminates.
