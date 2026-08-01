Returns whether this font's `cmap` covers every meaningful character of
`text` — i.e. each maps to a non-zero (non-`.notdef`) glyph id.

ASCII whitespace and control characters are treated as always covered, so
a run like `"ab cd"` is not rejected for its space. Bytes that fail to
parse (or lack a `cmap`) cover nothing.
Whether this font's `cmap` maps `character` to a real (non-`.notdef`)
glyph.
