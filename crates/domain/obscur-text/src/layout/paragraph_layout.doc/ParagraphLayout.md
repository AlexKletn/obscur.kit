A fully laid-out multi-font paragraph: its [`ParagraphLine`]s stacked
vertically together with the computed total [`size`](Self::size), alignment,
and overflow policy (spec §5.10).

This is the multi-font companion of [`TextLayout`](super::TextLayout): where
that wraps a single itemized [`GlyphRun`](crate::shaping::GlyphRun),
`ParagraphLayout` wraps a sequence of font-tagged [`ShapedRun`]s (as produced
by the font-fallback orchestrator) into lines whose segments may each use a
different font. The paragraph's `size` is derived from its lines: the width is
the widest line's width and the height is the sum of the line heights. Because
it embeds `f32` metrics and non-`Eq` glyph runs it derives `PartialEq` but not
`Eq`.
