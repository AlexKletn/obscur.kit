Whether to blank every string the UI renders (`OBSCUR_NO_TEXT=1`).

A glyph probe. The tree, the node count, the classes and the layout stay
**identical** — only the text content becomes empty, so no glyph is shaped or
rasterized. The difference in `present` is therefore the cost of the text alone,
with nothing else moving. Vello draws glyphs as filled paths rather than from an
atlas, and this list carries ~230 runs a frame.
