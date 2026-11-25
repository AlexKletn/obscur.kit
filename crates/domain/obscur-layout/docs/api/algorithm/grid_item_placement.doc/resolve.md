Runs the placement algorithm over `children`.

`flow` is the container's `grid-auto-flow`; `templ_cols_len` and
`templ_rows_len` are the explicit track counts the template expanded to
(an auto-repeat is already resolved by then), against which definite
lines and spans grow the grid into implicit tracks.
