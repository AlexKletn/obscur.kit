Every scroll container's live vertical offset (CSS Overflow 3 §scrolling),
together with the viewport-relative projection of [`boxes`](Self::boxes)
derived from it that [`layout_boxes`](Self::layout_boxes) exposes.

One field rather than the three it replaces, because the projection is a
cache of the offsets — see [`ScrollOffsets`], which is why no writer can
move an offset and leave the projection describing the previous one.
