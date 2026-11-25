The `(track, thumb)` border-box rectangles of a scroll container's vertical
scrollbar (CSS Overflow 3 §scrollbars) — the geometry `scrollbar_commands`
paints, factored out so the DevTools tree can project the SAME two part rects
(spec §4.16 "scrollbar parts") without re-deriving them. Returns `None` when
the geometry is degenerate (a non-positive scrollport, or content that does
not exceed the scrollport), so a non-overflowing container has no scrollbar
parts anywhere.
