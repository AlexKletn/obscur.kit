A single laid-out visual line of a multi-font paragraph: the font-tagged
segments it contains plus the geometry placing it within the paragraph
(spec §5.10).

Unlike [`TextLine`](super::TextLine), whose runs share a single font, a
`ParagraphLine` may span several faces: its [`segments`](Self::segments) are
the contiguous, same-font [`ShapedRun`]s of the line in visual (pen) order, so
a line that crosses a font boundary holds two segments. The geometry mirrors a
text line: [`top`](Self::top) is the y of the line box's top edge within the
paragraph, [`height`](Self::height) its box height,
[`baseline`](Self::baseline) the baseline offset measured down from `top`, and
[`x_offset`](Self::x_offset) the horizontal start offset produced by alignment.
Because it embeds `f32` metrics and non-`Eq` glyph runs it derives `PartialEq`
but not `Eq`.
