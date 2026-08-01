First free `(col, row)` in row-major order for a fully-auto block (first-fit).

This scans from the grid origin every call — the *dense* row-major search
(`grid-auto-flow: row dense`), which backfills earlier holes.
