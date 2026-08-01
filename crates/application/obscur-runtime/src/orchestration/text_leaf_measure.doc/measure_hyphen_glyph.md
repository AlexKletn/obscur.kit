Shapes a single hyphen (`-`) through `paragraph` at `font_size` and returns
its first glyph, for [`ParagraphLayout::wrap_hyphenated`] to render at a
break (spec §5.10).

The hyphen is shaped with the same [`measure_face`] the scene coordinator's
default face mirrors, so the glyph the measure pass sizes with is identical
to the one the compose pass paints, keeping the box height and painted line
count in agreement. Returns [`None`] only if shaping yields no glyph.
