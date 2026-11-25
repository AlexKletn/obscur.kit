Maps a `grid-auto-flow` value to its reserved [`InternedSymbol`].

This is a *multi-keyword* property: the axis (`row` / `column`) and the
optional `dense` packing keyword may appear together in either order. The
input is ASCII-lowercased and split on whitespace, then each accepted
combination folds to a single reserved handle:

* `row` → [`GRID_AUTO_FLOW_ROW`] (the default)
* `column` → [`GRID_AUTO_FLOW_COLUMN`]
* `dense` (bare) → [`GRID_AUTO_FLOW_ROW_DENSE`] (`row` is the implied axis)
* `row dense` / `dense row` → [`GRID_AUTO_FLOW_ROW_DENSE`]
* `column dense` / `dense column` → [`GRID_AUTO_FLOW_COLUMN_DENSE`]

Returns [`None`] for any other combination.
