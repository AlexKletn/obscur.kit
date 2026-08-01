The caret's pen x **within** a wrapped `line`: the sum of the line's own
spaced/tabbed glyph advances whose source **cluster** falls before `index`
(CSS Text 3 §8 letter-/word-spacing + §4.3 tab-size), measured from the
line's content start.

These are the SAME advances the segments are painted from — the line's
glyphs already carry the `apply_spacing` / `apply_tab_stops` adjustments the
compose pass wrapped them with, and any per-line trailing-tracking hang and
justification the wrapper applied (CSS Text 3 §8.1 / §7.1) — so a caret x
derived here lands exactly on the painted glyph edge, not on a fresh
spacing-blind re-shape. Because the line holds only glyphs with clusters in
its own byte range, filtering `cluster < index` yields the line-relative pen
x directly (no `\n`/preceding-line subtraction). A caret at the line's end
byte sums every glyph → the line's used content width. Single-line,
left-to-right: RTL reordering is not modeled (a documented approximation).
