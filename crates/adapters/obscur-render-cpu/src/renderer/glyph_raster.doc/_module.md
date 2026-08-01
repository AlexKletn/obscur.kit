Rasterizing a pre-shaped [`GlyphRun`] on the software backend — real outlines
where the run's font is registered, box-coverage cells where it is not (spec
§5.2, §5.18 synthetic styles).

Shaping already ran in the domain, so nothing here re-shapes: the run's glyph
ids, advances and positions are authoritative and both backends must agree on
them. What this module owns is the step after shaping — turning each positioned
glyph into filled coverage — and it owns it as one entity because the *two* ways
of doing that are one decision with one shared vocabulary: pen position,
baseline, advance cell, and the §5.18 synthesis hints that must apply the same
way down either branch. A run that leans in outline mode has to lean in fallback
mode too, or a host that registers a font sees italics move.

The interface is one call, [`draw_text_run`]. The internals are the registry
lookup and sfnt validation, `skrifa` outline extraction through
[`GlyphPathPen`], the faux-oblique shear, the faux-bold stroke, and the
whitespace convention the fallback cell obeys.
