Sets `c`'s offset to `px`, clamped to `0 ..= content_height −
scrollport_height` (CSS Overflow 3 §scrolling — the position is clamped to
the scrollable overflow rect), and refolds the projection against `fold`.

A non-finite `px` is ignored: nothing is written and nothing is refolded,
so a `NaN` cannot reach either the map or an observer's coordinates.
