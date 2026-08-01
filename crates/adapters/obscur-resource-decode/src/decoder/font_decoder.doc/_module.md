A font [`ResourceDecoder`] backed by `ttf-parser` (spec §5.18).

[`FontDecoder`] validates encoded TTF/OTF bytes by parsing them into a
`ttf_parser::Face` — proving the sfnt tables (`head`, `hhea`, `maxp`, ...)
are well-formed and the metrics readable — and then keeps the original,
validated face bytes as the shared CPU representation. That is the natural
representation for a font: `ttf-parser` (and the text/shaping stack above it)
reads a `Face` zero-copy directly from these bytes, so re-wrapping them costs
nothing while the parse guarantees the downstream face will not fail (spec
§5.18 shaping/glyph caching).
