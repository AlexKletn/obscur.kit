Which cell every grid item lands in (css-grid-2 §8.5, a subset).

The *Grid Item Placement Algorithm* is a phase of its own: it reads only the
items' authored [`GridPlacement`](super::GridPlacement)s, the container's
`grid-auto-flow` and the explicit track counts, and it answers with a table
of `(column, row, column-span, row-span)` plus the column and row counts the
grid ended up needing — implicit tracks included. Nothing in it measures,
sizes or positions anything; track sizing runs afterwards, on its output.

Fully definite items reserve cells first. Items locked along the flow axis
are placed next, followed by the remaining items. Both passes use CSS order
groups with source order breaking ties. Sparse flow
keeps a cursor, and dense flow starts each search at the origin. Column flow
uses the same algorithm with transposed axes.
