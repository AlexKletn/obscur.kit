Splits one line's logical-order `glyphs` into contiguous [`ShapedRun`]
segments homogeneous in font, direction, bidi level, font size, and synthesis
(faux bold / oblique), then reorders those segments into visual
(left-to-right) order via the UAX#9 L2 rule (see [`reorder_visual`]). Each
segment's [`GlyphRun`] takes its direction and font size from its glyphs'
source run, a `source` byte range from its glyphs' clusters (its last
logical segment ending at `line_end`), and carries its bidi `level`.

A boundary in ANY of {font, direction, level, font-size, synth-bold, synth-oblique}
ends a segment. Per CSS Inline Layout 3 §3.1 an inline box is split into
fragments at style boundaries, and CSS Text 3 §1.3 defines a run as a
maximal sequence of characters sharing formatting: glyphs that differ in
used font size or in synthesized boldness/obliqueness (CSS Fonts 4 §2.4,
spec §5.18) belong to distinct inline boxes and must not be coalesced into
one run rendered at the first glyph's size / synthesis. RTL clusters are restored to visual order within each line segment. The
glyphs sharing a cluster retain the order emitted by the shaper.
