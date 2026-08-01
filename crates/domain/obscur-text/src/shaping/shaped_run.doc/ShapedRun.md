A [`GlyphRun`] tagged with the [`FontId`] its glyph indices belong to
(spec §5.10).

A [`GlyphRun`]'s glyph ids are indices *into a specific font face*, but the
run itself does not record which face. Font fallback shapes one string with
several faces, so the result is a `Vec<ShapedRun>`: each element pairs a run
with the [`FontId`] its glyphs must be looked up and rasterized against. This
keeps the multi-font result expressible without changing [`GlyphRun`] itself.

The run also carries its source item's bidi embedding `level` (spec §5.10,
UAX#9): even levels are left-to-right, odd right-to-left. Line layout uses it
to reorder a line's segments into visual order (the L2 rule); `new` defaults
it to `0` (LTR) so callers that don't yet know the level keep compiling.

Because [`GlyphRun`] embeds `f32` metrics this type derives `PartialEq` but
not `Eq`.
The run also carries two per-run synthesis hints
([`synth_oblique`](Self::synth_oblique), [`synth_bold`](Self::synth_bold)):
when the font resolver matched the run to an upright/lighter face because no
true italic/bold cut was registered, the renderer must shear/thicken the
glyphs to fake the requested style (spec §5.18). Both default to `false`;
clusters and glyphs are unchanged — synthesis is purely a render-time effect.
