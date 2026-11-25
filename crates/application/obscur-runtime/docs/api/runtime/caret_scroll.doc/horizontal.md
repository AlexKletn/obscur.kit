Recomputes the focused single-line field's horizontal scroll offset so its
caret stays within the field's visible content width `W` — the standard
single-line text-control scroll model (CSS Overflow 3 §3.1
scroll-into-view; the text-control behaviour a `<input type=text>` scrolls
with). Called from the runtime's `after_edit`, so every path that
moves the caret (typing, delete, arrows, Home/End, click, drag-select,
select-all, undo/redo, IME commit) keeps the caret visible.

Only a **single-line** (non-wrapping) field scrolls horizontally: a
wrapping field (`white-space: normal`/`pre-wrap`/`pre-line`) is pinned to
`scroll_left = 0` (its overflow is handled by wrapping / a later
vertical-scroll item). `W` is the field's border-box width minus its
left/right padding + border (its content-box width, the run's scrollport).

The runtime measures the visible caret edges from the field's content-box
origin using the same paragraph layout as paint, including the text box's
position and the line's alignment offset. It shifts the visible interval
just enough to fit the caret and bounds the shift by the line's full visual
extent. It recalculates after layout when the text box changes.

Returns the offset to seat, or [`None`] to leave `scroll_left` unchanged —
which is the answer when the field has no text leaf or no retained box yet.
A field whose value fits (`content_width ≤ W`) resolves to `scroll_left = 0`
(byte-identical, unscrolled paint).
