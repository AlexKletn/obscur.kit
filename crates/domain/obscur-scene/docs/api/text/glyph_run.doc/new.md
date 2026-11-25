Creates a run from its positioned `glyphs`, the `font_size` they were
shaped at, and the source `text` they were produced from.

The [`font`](Self::font) handle defaults to `0` (the primary face); tag a
non-primary run with [`with_font`](Self::with_font).
