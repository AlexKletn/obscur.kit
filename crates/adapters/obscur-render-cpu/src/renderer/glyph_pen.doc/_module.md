A [`skrifa`] [`OutlinePen`] that accumulates a glyph's scaled outline into a
[`tiny_skia::Path`], placed at the glyph's device-space pen position (spec
§5.2 real glyph rasterization).

`skrifa` emits outline coordinates already scaled to pixels (via
[`Size`](skrifa::instance::Size)) in the font's y-up, baseline-at-origin
coordinate system. This pen flips the y axis and translates each point onto
the glyph's baseline pen position `(pen_x, baseline_y)` in the pixmap's
y-down device space, so the resulting path is ready to fill directly.
