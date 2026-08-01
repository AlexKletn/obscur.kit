Whether a `rspan × cspan` block anchored at `(row, col)` is entirely free,
treating any cell past the current occupancy extent (in either dimension) as
free. Unlike [`block_free`] this does **not** bound the column axis, so it
suits column-flow placement into implicit (growing) columns.
