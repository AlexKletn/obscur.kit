Which cell every grid item lands in (css-grid-2 §8.5, a subset).

The *Grid Item Placement Algorithm* is a phase of its own: it reads only the
items' authored [`GridPlacement`](super::GridPlacement)s, the container's
`grid-auto-flow` and the explicit track counts, and it answers with a table
of `(column, row, column-span, row-span)` plus the column and row counts the
grid ended up needing — implicit tracks included. Nothing in it measures,
sizes or positions anything; track sizing runs afterwards, on its output.

That narrow interface is what makes it one entity, because its internals are
a vocabulary the caller never has to learn: an occupancy grid of `bool`
cells, a two-phase pass over the items (definite lines first, then the
fully-auto ones), a placement cursor that moves forward for a *sparse* flow
and is reset to the origin for a *dense* one, and a first-fit search that
comes in a row-major and a column-major spelling. Those eight functions are
that machinery; [`PlacedItems`] and [`PlacedItems::resolve`] are the whole of
what [`GridLayout`](super::GridLayout) sees.
