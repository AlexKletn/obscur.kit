Maps a content-space `point` into the **unscrolled layout space** the boxes
and the hit-test index are keyed by, by adding back every crossed
container's `scroll_top`.

Returns the point unchanged when the frame has no scroll container at all,
so a document that never scrolls hit-tests byte-identically. Unlike
[`scrollable_at`](Self::scrollable_at) this does **not** filter on
`overflow_scrolls`: a `hidden` container still translates its content, so a
click inside one must still be un-translated.
