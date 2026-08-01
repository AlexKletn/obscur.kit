Draw a decoded raster [`SceneImage`], placed and (optionally) tiled within
`region`. A LEAF command (no nesting effect).

This is the CSS background placement model (`background-position` /
`background-size` / `background-repeat`):

* `tile` is the rectangle of **one** copy of the image, in absolute scene
  coordinates — its position (`background-position`) *and* size
  (`background-size`) already resolved by the coordinator. The image's
  intrinsic `width × height` is scaled to `tile`'s size and drawn at
  `tile`'s origin.
* `region` is the background paint area (the node's border box). It acts as
  a **clip**: no image pixel paints outside it.
* `repeat_x` / `repeat_y` tile the image along that axis, stepping by
  `tile`'s width / height from `tile`'s origin (both directions) until
  `region` is covered.

A plain full-box stretch is `tile == region` with `repeat_x == repeat_y ==
false` — visually identical to a stretch-to-`region` draw. A renderer
composes this with any active transform / clip / layer exactly like every
other leaf paint. The image carries its own [`AlphaMode`](crate::color::AlphaMode),
so each backend converts to its native premultiplied form honestly and both
show the same picture (spec §5.16).
