Fills the glyph run once using the outlines extracted from `font` at the
run's `font_size` (spec §5.2 real glyph rasterization).

For each glyph the outline is drawn (unhinted, no variation) into a
[`GlyphPathPen`] anchored at the baseline pen position
`(origin.x + glyph.x, origin.y + glyph.y)`; `skrifa` yields pixel-scaled,
y-up coordinates that the pen flips into the pixmap's y-down device space. A
glyph absent from the font, with no contours (whitespace, glyph 0), or whose
outline `skrifa`/`tiny-skia` rejects contributes nothing.

`skew` shears the outline to synthesize an oblique slant (`0.0` = upright);
synthetic bold expands the outlines before they join the same fill (spec §5.18).
