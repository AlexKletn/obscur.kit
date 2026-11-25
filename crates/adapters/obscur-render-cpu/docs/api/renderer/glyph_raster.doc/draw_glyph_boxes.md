Rasterizes `run` as one solid coverage cell per visible glyph — the fallback
used when no real font is registered for the run's handle (spec §5.2
rasterization approximation).

Each glyph fills the box whose top-left is `origin + (glyph.x, glyph.y -
font_size)` (the baseline sits `font_size` below the glyph's ascent) sized
`advance × font_size`, honoring `clip`. Whitespace glyphs — identified by the
mock shaper's convention that a glyph id is the Unicode scalar — and
non-positive cells draw nothing.

`skew` shears the cell into a forward-leaning parallelogram to approximate a
synthetic oblique (spec §5.18): the top edge (a full `font_size` above the
baseline) slides right by `skew * font_size` while the baseline edge stays
put. The cells are collected into one fill so overlaps composite once.
