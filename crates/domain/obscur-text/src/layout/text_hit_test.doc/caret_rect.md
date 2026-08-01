Computes the caret geometry for the cluster boundary at `byte_offset` on
line `line_index` (spec §5.10 caret).

Returns a thin `caret_width`-wide [`Rect`] spanning the line's vertical box,
positioned at the x of the boundary: the pen position just before the first
glyph whose cluster is `>= byte_offset`, or the line's trailing edge when no
such glyph exists. Returns `None` when `line_index` is out of range. This is
the inverse of [`hit`](Self::hit) and is used to place the §5.12
candidate window.
