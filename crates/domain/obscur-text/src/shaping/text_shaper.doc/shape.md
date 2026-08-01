Shapes `request` into a [`GlyphRun`] (spec §5.10).

Infallible: an unshapable character yields a `.notdef` (tofu) glyph, never
an error. The returned run's glyphs are in pen order.
