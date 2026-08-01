First free `(col, row)` in **column-major** order at or after the cursor
`(start_col, start_row)` — the column-flow search (`grid-auto-flow: column`).

Columns grow implicitly; `n_rows` is the fixed row-wrap bound. The current
column is scanned from `start_row`; each later column is scanned from row `0`.
Passing `(0, 0)` yields the dense column search (search from the origin every
call); passing the running cursor yields the sparse one.
