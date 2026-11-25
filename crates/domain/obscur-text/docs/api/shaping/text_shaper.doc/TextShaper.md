The hexagonal port for turning styled text into positioned glyphs
(spec §5.10).

A shaper takes a [`ShapingRequest`] (one uniformly-styled, already-itemized
run) and returns a [`GlyphRun`]. Real implementations wrap a shaping engine
(`HarfRust`) in an adapter; the domain stores whichever implementation it is
given as `Box<dyn TextShaper>`, so the trait is deliberately **object-safe**
— no generic or `Self`-returning methods.

Shaping is **infallible**: there is no error path. A character the font
cannot render still produces a glyph — the font's `.notdef` / "tofu" glyph
(glyph 0) — rather than a failure, so a run always yields glyphs for its
input.
