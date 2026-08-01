Builds a template from the authored rows of cell tokens.

Each row is a slice of cells; a cell is `Some(name)` for a named area or
[`None`] for a null (`.`) cell. Returns [`None`] when the grid is empty,
when the rows are ragged (unequal column counts), or when any named area is
not a filled rectangle (a hole, or two disjoint blocks of the same name).
