Resolves the placed tile rect for an element's image `content` within its box
`region`, per `object-fit`, centered (the `object-position: 50% 50%` default).

The tile size comes from [`resolve_object_fit_size`] against the image's
intrinsic logical size and the box size; the tile is then centered within the
region so `contain` letterboxes and `cover` overflows symmetrically (the
`region` passed to `DrawImage` clips the overflow).

The intrinsic size is the image's [`logical_size`](SceneImage::logical_size),
not its pixel count: an `@2x` DPI variant stands in for a box half its pixel
dimensions, so `object-fit: none` must place it at that logical size and let
the root device-scale transform map it back to 1:1 physical pixels — not draw
it at double size (spec §5.16).
