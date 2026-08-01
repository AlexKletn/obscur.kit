Blits a decoded [`SceneImage`] as one copy scaled to `tile`, clipped to
`region`, and optionally tiled along the `repeat_x` / `repeat_y` axes to fill
`region`, honoring the active `clip` and `transform` (spec §5.16).

`tiny-skia` stores **premultiplied** RGBA, so a [`AlphaMode::Straight`] image is
premultiplied per pixel here (a [`AlphaMode::Premultiplied`] image is used as-is)
before being wrapped into a source [`tiny_skia::Pixmap`], once, and reused for
every tile. Each copy is drawn with [`draw_pixmap`](tiny_skia::Pixmap::draw_pixmap)
under a transform that scales the image's intrinsic `width × height` to `tile`'s
size and translates it to the copy's origin (stepping by `tile`'s width/height on
the repeated axes), composed with the running `transform`. When tiling — or when
a single tile overflows `region` — the draw is confined to `region` by
intersecting a `region` clip mask into the active `clip`. The default nearest
sampling keeps the blit crisp and deterministic. A zero-sized image, tile, or
region, or a source pixmap that fails to allocate, draws nothing.
