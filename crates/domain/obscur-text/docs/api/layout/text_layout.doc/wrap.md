Greedily wraps a single shaped `run` of `text` into a paragraph no wider
than `max_width` (spec §5.10).

The algorithm walks the run's glyphs, grouping them into words at the
[`LineBreak::opportunities`] of `text`. Words are packed onto the current
line until adding the next word would exceed `max_width` while the line is
non-empty, at which point the line is closed and the word starts the next
one; a mandatory (hard) break always closes the current line. A word wider
than `max_width` still occupies its own line rather than being split.

Each line is stacked at a cumulative `top` (spaced by `line_height`), given
the shared `baseline`, and offset horizontally by
[`TextAlignment::x_offset`]. When `max_width <= 0` (an unconstrained width)
or `text` has no break opportunities, the whole run becomes a single line.

This greedy wrapper assumes a **single itemized run** whose glyph clusters
are monotonic byte offsets — as produced by the deterministic mock shaper —
and does not perform bidi reordering or per-glyph justification.
