Resolves a within-line pen x (`local_x`, measured from the `line`'s content
start with any alignment offset already undone) to the nearest cluster byte
on that line — the per-line **inverse** of [`line_caret_x`]
over the SAME spaced/tabbed glyph advances (spec §5.11). The boundary whose
horizontal midpoint `local_x` falls at or before is chosen (a tie resolves to
the earlier boundary); a `local_x` past the last glyph clamps to `line_end`.
