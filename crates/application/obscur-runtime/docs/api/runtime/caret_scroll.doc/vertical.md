Recomputes the focused **multiline** field's (`<textarea>`) vertical scroll
offset so its caret's LINE stays within the field's visible content height
`H` — the VERTICAL analogue of [`horizontal`](Self::horizontal)'s
horizontal single-line scroll (CSS Overflow 3 §3.1 scroll-into-view). Called
from the runtime's `after_edit`, so every path that moves the caret
(typing, `Enter`, delete, arrows incl. ArrowUp/Down, Home/End, click,
drag-select, select-all, undo/redo, IME commit) keeps the caret's line
visible.

This drives the **same** per-node `scroll_top` the overflow scroll container
uses (the wheel + scrollbar move the same value), so the widget's `scroll_top`
map is the single source of truth: typing scrolls the container, and the
caret-follow, wheel, and scrollbar all move `scroll_top` (CSS Overflow 3
§scroll-container). Paint + hit-test already offset by it (the container's
`translate(0, −scroll_top)` brackets the field's subtree, so the editing run/
caret/selection move with it, and the pointer maps back through it), so this
method only has to place `scroll_top`.

Only a multiline field whose computed `overflow` scrolls (`auto`/`scroll`)
**and** whose height is a definite `px` length participates — that is exactly
a fixed-height `<textarea>` whose content overflows rather than growing the
box. A single-line field, a `hidden`-overflow field, or an auto-height field
that grows to fit is left at `scroll_top == 0` (byte-identical, unscrolled).

The clamp keeps the caret's line box within `[scroll_top, scroll_top + H]`,
`H` the field's content-box height: when the line top rises above the top
edge (`caret_top − scroll_top < 0`) `scroll_top` snaps to `caret_top`; when
the line bottom drops past the bottom edge (`caret_bottom − scroll_top > H`)
it snaps to `caret_bottom − H`. `scroll_top` is then clamped to
`0 ..= max(0, content_height − H)` so the field never scrolls past its
content (a field whose content fits resolves to `0`). Line geometry +
content height are measured from the live session text with the SAME wrapped
layout the caret paints from ([`SceneCoordinator::caret_line_scroll_geometry`]),
so paint and the scroll agree; the field/text boxes supply the definite
scrollport dimensions.

`stored` is the container's current offset; the return is the offset to
seat, or [`None`] to leave it alone.
