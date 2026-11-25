One shaped, positioned glyph in a [`GlyphRun`](super::GlyphRun): a glyph
identifier paired with the pen position it occupies (spec §5.10, §5.2).

This is the backend-independent unit a [`DrawText`](crate::command::PaintCommand::DrawText) command carries. Shaping
has already run (in the `obscur-text` domain), so a renderer never re-shapes:
it maps [`glyph_id`](Self::glyph_id) to an outline and draws it at
[`x`](Self::x)/[`y`](Self::y). Because both the GPU and the software backend
consume the very same positioned glyphs, they cannot diverge on text layout —
only on rasterization (spec §5.2 determinism boundary).

- [`x`](Self::x)/[`y`](Self::y) are the glyph's pen position **relative to the
  run's origin**, in logical pixels; `y` is measured at the text baseline.
- [`advance`](Self::advance) is the glyph's horizontal advance, letting a
  software backend that lacks real glyph outlines approximate the glyph with a
  coverage cell of this width.
- [`glyph_id`](Self::glyph_id) is a **font glyph index** — the same contract as
  the domain
  [`ShapedGlyph`](../../../obscur_text/shaping/struct.ShapedGlyph.html). A real
  shaper (the HarfRust adapter) emits real, font-specific glyph indices, so a
  renderer draws them directly with no remap; the reference
  [`MockTextShaper`](../../../obscur_text/shaping/struct.MockTextShaper.html)
  emits the Unicode scalar value as a placeholder index for headless
  determinism.
- [`cluster`](Self::cluster) is the byte offset into the run's source text the
  glyph maps back to (for caret hit-testing / accessibility).

Storing `f32` fields means the type derives `PartialEq` but not `Eq`.
