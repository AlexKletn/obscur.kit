The base device-pixel scale the UI renders at: the display's HiDPI
[`scale_factor`](Self::scale_factor) — **independent of the window size**.

The scale is *not* tied to the window width (no fit-to-width zoom): the
gallery renders at its natural CSS size for the display density, so widening
or narrowing the window changes the logical viewport and the layout reflows
(the gallery divides the available width), exactly like a real application —
rather than scaling the whole UI up and down. The user `zoom` composes on
top for an explicit magnify.
