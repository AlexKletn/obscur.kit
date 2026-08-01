Truncates one overflowing line's `glyphs` in place so it fits `max_width`
under the `overflow` policy, at glyph granularity (spec §5.10).

[`Clip`](TextOverflow::Clip) — and [`Ellipsis`](TextOverflow::Ellipsis)
with no `ellipsis_glyph` — keeps the largest prefix whose cumulative
advance is `<= max_width` and drops the rest, adding nothing.
[`Ellipsis`](TextOverflow::Ellipsis) with a glyph reserves that glyph's
advance, keeps the largest prefix whose advance plus the ellipsis fits
`max_width`, then appends the ellipsis glyph. The appended glyph carries the
last kept glyph's cluster and inherits its font, direction, and bidi level
(falling back to the line's first glyph when nothing fits) so it segments
and reorders with the kept prefix. A partially-over glyph is dropped, so the
resulting line width never exceeds `max_width`.
