Splits one line's logical-order `glyphs` into contiguous [`ShapedRun`]
segments homogeneous in font, bidi level, font size, and synthesis
(faux bold / oblique), then reorders those segments into visual
(left-to-right) order via the UAX#9 L2 rule (see [`reorder_visual`]). Each
segment's [`GlyphRun`] takes its direction and font size from its glyphs'
source run, a `source` byte range from its glyphs' clusters (its last
logical segment ending at `line_end`), and carries its bidi `level`.

A boundary in ANY of {font, level, font-size, synth-bold, synth-oblique}
ends a segment. Per CSS Inline Layout 3 §3.1 an inline box is split into
fragments at style boundaries, and CSS Text 3 §1.3 defines a run as a
maximal sequence of characters sharing formatting: glyphs that differ in
used font size or in synthesized boldness/obliqueness (CSS Fonts 4 §2.4,
spec §5.18) belong to distinct inline boxes and must not be coalesced into
one run rendered at the first glyph's size / synthesis. Within a segment
the glyph order is left as the shaper produced it: both HarfRust and the
deterministic mock already emit an RTL run's glyphs in visual order.
