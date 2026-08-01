Advances every preserved tab (U+0009) in already-shaped `runs` over the source
`text` to the next `tab_width` tab stop (§4.3), returning the runs with the tab
glyphs' advances rewritten.

`tab_width` is the resolved tab-stop width in logical pixels (`tab-size` as a
`<number>` times the space advance, or a `<length>` verbatim). Runs are walked
in logical order with a running pen measured from the line start; the pen is
reset after each preserved `\n`. Only the advance of a tab glyph changes —
glyph identity, cluster, offsets, font, direction, level, and synthesis are
preserved.

The pass early-returns `runs` unchanged when `tab_width <= 0` or `text`
contains no tab, so an unauthored or collapsing-mode text node is completely
unaffected (byte-for-byte identical).
