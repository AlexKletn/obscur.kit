A fully laid-out paragraph: its [`TextLine`]s stacked vertically together with
the computed total [`size`](Self::size), alignment, and overflow policy
(spec §5.10).

The paragraph's `size` is derived from its lines: the width is the widest
line's width and the height is the sum of the line heights. Because it embeds
`f32` metrics and non-`Eq` glyph runs it derives `PartialEq` but not `Eq`.
