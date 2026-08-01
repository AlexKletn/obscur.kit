Attempts to break this word so its head (plus the hyphen glyph's advance)
fits within `limit`, returning the head glyphs (with an appended
`hyphen_glyph` when supplied) and the remaining tail as a fresh word.

The **last** (widest) fitting opportunity is chosen. Returns [`None`] when
no opportunity's head fits, so the caller falls back to placing the whole
(overflowing) word.
