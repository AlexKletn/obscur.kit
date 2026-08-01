The shaped output of one run: its ordered glyphs plus the direction, font
size, and source byte range they were produced from (spec §5.10).

A `GlyphRun` is what a [`TextShaper`](crate::shaping::TextShaper) returns for
a single [`ShapingRequest`](crate::shaping::ShapingRequest). The glyphs are
stored in visual (pen) order; [`source`](Self::source) records the byte range
of the original text the run covers, so callers can map glyphs back to the
buffer they came from. Because it embeds `f32` metrics it derives `PartialEq`
but not `Eq`.
