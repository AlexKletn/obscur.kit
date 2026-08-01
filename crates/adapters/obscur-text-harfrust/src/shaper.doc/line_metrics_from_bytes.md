Reads a font's real vertical [`LineMetrics`] from its `head` (units-per-em)
and `hhea` (ascender / descender / line-gap) tables, scaled to `font_size`.

Metrics are stored in font design units; scaling by `font_size / upem`
converts them to logical pixels. `hhea.descender` is negative below the
baseline, so its magnitude is taken to match [`LineMetrics`]'s positive
descent convention. Returns [`None`] when the bytes are not a loadable font,
the `head`/`hhea` tables are missing, or `units_per_em` is zero.
