Shapes a single space (U+0020) through `paragraph` at `font_size` and returns
its total advance, the unit a `<number>` `tab-size` multiplies (§4.3).

Shaped with the same [`measure_face`] the scene coordinator mirrors, so the
space width the measure pass resolves the tab stop from is identical to the
compose pass's. Returns `0.0` when shaping yields no glyph.
