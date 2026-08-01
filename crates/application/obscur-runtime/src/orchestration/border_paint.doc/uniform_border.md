The single width and color of a node's border when it is **uniform**, else
`None`.

A border is uniform when all four sides share one positive pixel width, one
color, and the `solid` style. Only such a border can be painted as a single
even-odd rounded ring (a concentric inner rounded rect), so a non-uniform
border (differing widths / colors / styles, or any non-`solid` side) returns
`None` and keeps the per-side four-rect approximation.
