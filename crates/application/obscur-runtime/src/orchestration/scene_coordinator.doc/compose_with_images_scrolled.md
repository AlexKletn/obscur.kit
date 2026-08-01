Composes like [`compose_with_images`](Self::compose_with_images), additionally
applying each scroll container's live scroll offset and painting its
scrollbar (CSS Overflow Module Level 3 §scroll-container, §scrollport).

`vectors` maps an element `src` to its parsed [`SceneVector`] — the
vector-content counterpart of `images`. A `src` present in `vectors` paints
as `DrawPath` outlines placed by `object-fit`, with `currentColor` resolved
against the node's computed `color` (spec §5.17); a `src` absent from it
paints nothing, exactly like an unloaded image. An empty map reproduces the
image-only behaviour byte-for-byte.

`scroll` maps a scroll-container [`NodeId`] to its
[`ScrollPaint`] (current `scroll_top` + scrollable `content_height`). For a
node present in the map the coordinator brackets its descendants — inside
the node's own `overflow` clip and outside the node's own paint — with a
`PushTransform(translate(0, −scroll_top))`, so the content scrolls within
the (clipped) scrollport while the container's own background/border stay
put; and paints a vertical scrollbar (track + thumb) on the right, on top of
and unaffected by the scroll. A node **absent** from the map contributes
neither, so an **empty** map reproduces
[`compose_with_images`](Self::compose_with_images) byte-for-byte. The scroll
transform is elided at `scroll_top == 0` (it reduces to the identity), so an
unscrolled container differs from the pre-scroll list only by its scrollbar.
