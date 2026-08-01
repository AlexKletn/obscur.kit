The number of rows a `<VirtualFor>` realizes for a viewport of
`viewport_height` logical pixels scrolled to `scroll_y`, given an
estimated row `extent` (spec §4.9).

It is the length of the materialized range
[`VirtualLayout`](obscur_layout::virtualization::VirtualLayout) computes
for the viewport — the visible band widened by overscan — so it grows with
the viewport and shrinks as rows get taller, and is **independent of how
many items the collection holds**. A `<VirtualFor>`'s `each={…}` carries no
compile-time length, so the layout is queried over an assumed-large
collection: virtualization is exactly the regime where the collection
overflows the viewport, and the window is bounded by the viewport, not by
that count. This is what a `<For>` — one node per item — cannot promise.
