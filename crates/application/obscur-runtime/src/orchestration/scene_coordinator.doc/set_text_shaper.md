Installs `shaper` as the inner text shaper subsequent shaping runs through,
replacing the previous one and keeping the itemizer and resolver in place.

The engine is wrapped in a **fresh** shaping cache so every shape call
(measure, arrange, compose) is memoized behind one seam and no run shaped
by the previous engine survives. Installing a different shaper changes the
emitted [`GlyphRun`](obscur_scene::text::GlyphRun) — that is the seam the
runtime's `set_text_shaper` drives (e.g. the HarfRust adapter for real,
font-specific glyph indices).
