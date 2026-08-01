Builds the paint commands for a scroll container's vertical scrollbar — a
track filling the right gutter of the `scrollport` plus a thumb whose length
and position reflect `scroll_top` and `content_height` (CSS Overflow 3
§scrollbars).

The thumb length is `scrollport_h · scrollport_h / content_height` (the
visible fraction of the content), clamped to at least [`SCROLLBAR_MIN_THUMB`],
and its top is `scroll_top / (content_height − scrollport_h)` of the free
track. Returns an empty list when the geometry is degenerate (a non-positive
scrollport, or content that does not exceed the scrollport), so a
non-overflowing container paints no scrollbar. The thumb is **not**
interactive-draggable in this pass — the wheel drives `scroll_top`; drag is a
documented follow-up.
