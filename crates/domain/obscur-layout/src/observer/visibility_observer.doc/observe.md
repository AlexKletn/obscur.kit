Records the newly-committed box and optional `viewport`, returning a
[`VisibilityChange`] on a visible/hidden transition (or on the first
observation).

The node is visible when its box area is greater than zero and, when a
`viewport` is given, the box overlaps it. Returns `None` when visibility
is unchanged.
