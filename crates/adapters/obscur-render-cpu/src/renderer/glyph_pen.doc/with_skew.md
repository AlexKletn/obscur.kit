Creates a pen that additionally shears the outline by `skew` to synthesize
an oblique slant (spec §5.18): each point's device x is shifted right by
`skew` times its scaled height above the baseline, so the glyph leans
forward. A `skew` of `0.0` is byte-identical to [`GlyphPathPen::new`].
