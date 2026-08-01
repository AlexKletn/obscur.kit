Reads a face's design metrics (units-per-em plus `hhea`
ascender / descender / line-gap) in font units, or `None` when the bytes are
not a loadable font, the `head`/`hhea` tables are missing, or `units_per_em`
is zero. `descent` keeps the font's signed (negative-below-baseline)
convention, matching [`FontMetrics`].

Only `hhea` is consulted: the OS/2 `USE_TYPO_METRICS` selection between
`sTypoAscender`/`usWinAscent` is **not** implemented, so a face that asks for
its typo metrics is still measured by its `hhea` ones.
