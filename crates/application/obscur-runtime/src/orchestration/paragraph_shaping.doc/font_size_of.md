Reads a node's computed `font-size` in logical pixels, defaulting to
[`DEFAULT_FONT_SIZE`] when absent or not an absolute pixel length.

This mirrors the layout coordinator's font-size resolution so the scene draws
each text run at the same size the layout phase measured its box with. Reading
only `px` is sufficient: the resolver computes `font-size` to an absolute pixel
length before any other property (it is the base every `em` folds against), so
`2rem` arrives here as `32px` rather than falling back to the default.
