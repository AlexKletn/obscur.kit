Shapes a single ellipsis (`…`) through `paragraph` at `font_size` and returns
its first glyph, for [`ParagraphLayout::wrap_hyphenated`] to append when
`text-overflow: ellipsis` truncates an overflowing line (spec §5.10).

The ellipsis is shaped with the same [`measure_face`] the scene coordinator's
default face mirrors, so the glyph the measure pass sizes with is identical to
the one the compose pass paints, keeping the box width and painted lines in
agreement. Returns [`None`] only if shaping yields no glyph.
