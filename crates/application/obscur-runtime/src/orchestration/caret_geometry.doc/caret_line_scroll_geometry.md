The vertical extent a **multi-line** field needs to scroll the caret into
view: `(line_top, line_height, content_height)` for byte `index`, all
measured DOWN from the text box origin over the SAME wrapped
[`ParagraphLayout`] the caret paints from (spec §5.10). `line_top` /
`line_height` are the caret's line box (add `origin.y` for the absolute
paint y); `content_height` is the wrapped text's total height (its last
line's bottom). The runtime feeds these to the VERTICAL caret
scroll-into-view (CSS Overflow 3 §3.1) that keeps a `<textarea>`'s caret
line within its visible content box — the vertical analogue of
[`caret_advance_x`]. A single-line / empty layout
resolves to line 0 (`top` 0, the metric line height) with a
single-line-tall content height.
