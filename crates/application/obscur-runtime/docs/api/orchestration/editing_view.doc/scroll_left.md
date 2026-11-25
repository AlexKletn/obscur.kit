The field's horizontal scroll offset (logical px, ≥ 0): how far the text
run is scrolled left within the field's content box so the caret stays in
view (the single-line text-control scroll model, CSS Overflow 3 §3.1
scroll-into-view). The compose phase translates the field's run, caret,
selection highlight and preedit by `−scroll_left`; the field's own
`overflow: hidden` clip then hides the scrolled-out head/tail. `0.0` for a
value that fits its box (byte-identical, unscrolled paint) and always `0.0`
for a wrapping (multi-line) field, which does not scroll horizontally.
