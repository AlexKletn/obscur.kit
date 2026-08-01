An in-memory map of rasterized glyphs keyed by a font+size+glyph hash.

Rasterizing a glyph run is costly and highly repetitive — the same glyph at
the same size recurs across frames and across text. This cache stores each
rasterized [`GlyphEntry`] under a caller-computed `u64` that folds together
the font identity, size, and glyph id. It holds no GPU resources and is fully
testable without a device.

This is deliberately a simple map: real glyph caching packs coverage into a
GPU atlas texture and tracks free rectangles. That atlas packing is a future
refinement; the key/entry contract here is the stable part callers depend on.
