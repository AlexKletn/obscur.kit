Shapes a single hyphen (`-`) at `font_size` through the paragraph
orchestrator and returns its first glyph, for
[`ParagraphLayout::wrap_hyphenated`] to render at a break (spec §5.10).

The hyphen is shaped with the coordinator's default face — identical to the
layout coordinator's `measure_face` — so the glyph the compose pass paints
matches the one the measure pass sized with. Returns [`None`] only if
shaping yields no glyph.
