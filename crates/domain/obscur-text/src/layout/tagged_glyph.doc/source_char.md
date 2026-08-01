The source character this glyph was shaped from: the character at the
glyph's cluster (a byte offset into the paragraph's `text`), if any.

A glyph whose cluster is not a character boundary of `text` — or is past
its end — has no source character, which is why this is fallible rather
than an index.
