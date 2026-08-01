Marks a `rspan × cspan` block occupied, growing `occ` to reach the block.

Rows are at least `n_cols` wide but grow to `col + cspan` so a *column-flow*
item placed past the explicit column count still marks its cells (implicit
columns). For row flow `col + cspan ≤ n_cols` always, so the width is exactly
`n_cols` and this is byte-identical to the previous fixed-width behaviour.
