A deterministic, reference [`TextShaper`](super::TextShaper) modeling an idealized monospace
font (spec §5.10).

`MockTextShaper` exists so headless tests have reproducible shaped output
without a real font engine. It emits exactly one glyph per Unicode scalar
value in the source text, with:

- `glyph_id` = the scalar's code point,
- `cluster` = the scalar's **byte offset** in the source text,
- `x_advance` = `font_size * ADVANCE_RATIO`, a fixed monospace em-ratio,
- zero placement offsets.

The glyphs are emitted in **visual** (pen, left-to-right) order, matching a
real shaper such as `HarfRust`: an [`TextDirection::LeftToRight`] request
yields glyphs in logical order (`glyph_id`s `['a','b','c']`, clusters
`[0,1,2]` for `"abc"`), while an [`TextDirection::RightToLeft`] request
yields the **reversed** sequence (`['c','b','a']`, clusters `[2,1,0]`). Each
glyph keeps its own source byte offset as its cluster — clusters stay the
source offsets, they simply appear in reversed order. This keeps the mock
visually honest so the bidi L2 line reordering (which trusts each run's
glyphs to already be in visual order) is consistent for the mock.

It deliberately does **not** perform ligatures, font fallback, or real font
metrics — every glyph is uniform width. Given identical requests it produces
identical [`GlyphRun`]s (no pointer hashing, no order-dependent iteration),
which is exactly the determinism headless golden tests rely on.
