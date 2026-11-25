The rectangle one named `grid-template-areas` area spans, in **1-based grid
line numbers** (spec §4.2, the named-areas subset).

A named area is a filled rectangle of cells; this records the four grid lines
that bound it. Line numbers are 1-based and the *end* edges are the line
*after* the last occupied track, exactly as CSS `grid-row` / `grid-column`
line placement reads: an area filling row 1, columns 1-2 (the `"h h"` first
row of a two-column grid) is `row_start = 1`, `row_end = 2`,
`column_start = 1`, `column_end = 3` — i.e. it can be assigned straight onto a
child's `grid-row: 1 / 2` and `grid-column: 1 / 3`.

# Determinism

`GridAreaRect` is a plain `Copy` value with no interior mutability, so an
area's placement resolves deterministically from the template alone.
