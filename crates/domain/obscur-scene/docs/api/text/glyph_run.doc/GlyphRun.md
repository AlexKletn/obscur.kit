The shaped output of one text run: its positioned glyphs plus the font size
and source text they were produced from (spec §5.10, §5.2).

A `GlyphRun` is the payload of a
[`DrawText`](crate::command::PaintCommand::DrawText) command. Text is shaped
**once**, in the `obscur-text` domain, and the resulting positioned glyphs are
carried here so every renderer consumes an identical run — restoring the
cross-backend determinism boundary (spec §5.2). Glyphs are stored in pen
order; each carries a position relative to the command's origin.

Each glyph's [`glyph_id`](PositionedGlyph::glyph_id) is a **font glyph index**
(the same contract as the domain `ShapedGlyph`), so a renderer draws it
directly against the host font with no remap.

The run also carries a [`font`](Self::font) handle: a plain `u32` index that
names *which* font face the run's glyph indices belong to. It is a bare index
(id `0` = the primary/first-choice face) rather than a typed identity so that
`obscur-scene` stays independent of `obscur-text`. Font fallback shapes one
paragraph across several faces and emits one `GlyphRun` per contiguous
same-font segment, so this handle is what lets a renderer pick the right font
for each run (spec §5.10).

[`text`](Self::text) retains the source string as metadata (accessibility and
diagnostics). Because it embeds `f32` metrics the type derives `PartialEq` but
not `Eq`.

# Synthetic styling

Two plain-`bool` render hints — [`synth_oblique`](Self::synth_oblique) and
[`synth_bold`](Self::synth_bold) — travel with the run: when the text's font
resolver had no true italic/bold cut and fell back to an upright/lighter face,
the renderer shears/thickens the glyphs to fake the requested style. They are
bare `bool`s (not an `obscur-text` type) so this crate stays independent of the
text domain; both default to `false`.
