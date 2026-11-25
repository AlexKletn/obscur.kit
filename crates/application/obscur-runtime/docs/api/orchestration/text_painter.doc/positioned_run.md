Converts one line [`segment`](ShapedRun) into a font-tagged scene [`GlyphRun`],
laying its glyphs out left-to-right into pen positions on the line's baseline.

The segment's glyphs carry only per-glyph advances, so the pen accumulates
them from the segment's left edge (`0`, relative to the segment origin the
caller places); each glyph's `y` is the line `baseline` (the ascent within the
line box), and the shaping `x_offset`/`y_offset` are added on top. `text` is
the source substring the segment covers, and the run is tagged with the
segment's [`font`](ShapedRun::font) index so the renderer can pick the right
face, plus the segment's synthetic-styling hints
([`synth_oblique`](ShapedRun::synth_oblique) / [`synth_bold`](ShapedRun::synth_bold))
so a faux italic/bold run is sheared/thickened at raster time (spec §5.18).
For a single-font line (one segment) this reproduces the original single-run
behavior exactly, with font `0` and no synthesis.

A glyph whose source character is a non-printing **control** (a C0/C1 control,
including the segment-break `'\n'` and `'\t'`) is dropped from the emitted run
so it paints no visible `.notdef` tofu box (CSS Text 3 §4.1.1 — a preserved
segment break is a zero-width forced break, not a rendered character). Its pen
advance is still accumulated, so every following glyph keeps its position and
the caret advance-walk (which sums every shaped glyph, control glyphs included)
stays in exact agreement — the suppression is purely visual, leaving byte
offsets, advances, and line breaks untouched. `content` is the full shaped
display string the segment's glyph clusters index into. Text with no control
characters is byte-identical to the pre-suppression run.
