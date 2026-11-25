Computes the mitered trapezoid **quads** painting a node's border, in top/
right/bottom/left paint order, each paired with its side color (spec CSS
Backgrounds and Borders §4).

A side contributes a quad only when its computed `border-*-style` is `solid`,
its `border-*-width` is a positive pixel length, **and** its resolved color is
not fully transparent; every other side (unauthored, `none`, zero-width, or a
transparent color) paints nothing, so a node with no authored border yields an
empty list (byte-identical to before borders were painted). Skipping
transparent sides is what makes the classic CSS border-triangle trick work.

Each side's quad is a **mitered trapezoid**: its outer edge spans the full
border-box side, and its inner edge is inset by the two adjacent border widths,
so the diagonals from each outer corner to the matching inner corner are the
45°-ish miters that split every corner between its two sides. Adjacent sides
therefore share a corner diagonal exactly (no overlap, no gap), which is the
correct CSS border-corner model. When a box collapses (a `0×0` content box, so
only the border widths remain) a side whose outer edge shrinks to a point
degenerates into a **triangle** — e.g. a lone `border-bottom` yields an
up-pointing triangle — exactly the arrow/marker glyph the border trick draws.
For a normal, single-color uniform border the four trapezoids tile the same
band the old four rects did (visually identical; corners now mitered rather
than square-joined). A side's color is its `border-*-color`, falling back to
the node's computed `color` (CSS `currentColor`) when the color is unset.

The four widths arrive as [`BorderWidths`], read once by the caller for all
three border shapes; this is the last of the three attempts, and re-reading
them here would be the third reading of the same four properties.
