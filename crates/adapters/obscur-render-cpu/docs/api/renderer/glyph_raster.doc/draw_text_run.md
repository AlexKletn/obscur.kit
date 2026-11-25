Rasterizes a pre-shaped [`GlyphRun`] at `origin`, drawing real glyph outlines
when the run's font is registered and box-coverage cells otherwise (spec §5.2).

The run's glyph positions are authoritative — shaping already ran in the
domain, so this never re-shapes. When [`GlyphRun::font`] names a font
registered via [`CpuRenderer::register_font`](crate::renderer::CpuRenderer::register_font)
and its bytes parse as an sfnt,
each glyph's outline is extracted with `skrifa`, scaled to `font_size`, placed
on its baseline pen position, and filled (see [`draw_glyph_outlines`]).
Otherwise — no font registered for the handle, or unparseable bytes — the run
degrades to the documented box-coverage fallback (see [`draw_glyph_boxes`]) so
device-free callers that register no font still get a defined, non-empty mark.
With a stop-less gradient brush (no resolvable color) the whole run is skipped.
