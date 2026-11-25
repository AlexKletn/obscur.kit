Collects a single glyph's outline into a device-space [`tiny_skia::Path`].

Constructed with the glyph's baseline pen position; each pen callback maps a
scaled font-space point `(x, y)` (y up from the baseline) to the device point
`(pen_x + x + skew * y, baseline_y - y)` (y down). The `skew` factor is the
synthetic-oblique shear (spec §5.18): with `skew == 0` the map is the plain
upright placement `(pen_x + x, baseline_y - y)`; a positive `skew` slides
above-baseline points rightward so the glyph leans forward, faking an italic.
[`GlyphPathPen::into_path`] finishes the builder, yielding `None` for a glyph
with no contours (whitespace, glyph 0, or a degenerate outline `tiny-skia`
rejects) so it draws nothing.
