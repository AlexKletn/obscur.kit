A scroll container's opening `PushTransform(translate(0, −scroll_top))`,
sorted *after* the node's own paint ([`PAINT_ORDER`](Self::PAINT_ORDER)) at
the same document position so the container's own background/border stays
**fixed** in the scrollport while its descendants (all at a strictly greater
`tree_index`) are translated by the scroll offset. Inside the overflow clip
(whose push is at [`CLIP_PUSH_ORDER`](Self::CLIP_PUSH_ORDER)) so scrolled
content is clipped to the scrollport (CSS Overflow 3 §scrollport). Any value
above `PAINT_ORDER` works — enter indices are unique per node.
