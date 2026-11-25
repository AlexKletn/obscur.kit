Flattens the font-tagged `runs` into one ordered stream of [`TaggedGlyph`]s,
preserving run order and tagging each glyph with its run's font,
direction, and font size.

Within an RTL run, reorder cluster groups to logical byte order for wrapping.
Glyphs sharing a cluster keep their shaping order.
