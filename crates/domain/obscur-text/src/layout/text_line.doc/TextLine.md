A single laid-out visual line: the glyph runs it contains plus the geometry
placing it within the paragraph (spec §5.10).

A line owns one or more [`GlyphRun`]s (a single run in the common,
already-itemized case), the source byte [`range`](Self::range) it covers, and
the metrics needed to position it: [`top`](Self::top) is the y of the line
box's top edge within the paragraph, [`height`](Self::height) its box height,
[`baseline`](Self::baseline) the baseline offset measured down from `top`, and
[`x_offset`](Self::x_offset) the horizontal start offset produced by alignment.
[`width`](Self::width) is the sum of the runs' advances and is computed at
construction. Because it embeds `f32` metrics and non-`Eq` glyph runs it
derives `PartialEq` but not `Eq`.
