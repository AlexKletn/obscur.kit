First free `(col, row)` in **row-major** order at or after the cursor
`(start_row, start_col)` — the *sparse* row-major search
(`grid-auto-flow: row`).

The current row is scanned from `start_col` (never earlier — the cursor only
moves forward, so a hole left behind is not backfilled); each later row is
scanned from column `0`. A span wider than the row wraps to the next row.
