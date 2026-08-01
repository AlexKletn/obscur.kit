Draws a decoded [`SceneImage`] as one copy scaled to `tile`, clipped to
`region`, and optionally tiled along the `repeat_x` / `repeat_y` axes to fill
`region` (spec §5.16, CSS Backgrounds §3).

The RGBA8 pixels are wrapped into a [`peniko::ImageData`] — zero-copy, since
the [`SceneImage`]'s `Arc<[u8]>` becomes the image [`Blob`] directly — tagged
with the image's own alpha encoding ([`AlphaMode::Straight`] →
[`ImageAlphaType::Alpha`], [`AlphaMode::Premultiplied`] →
[`ImageAlphaType::AlphaPremultiplied`]), so Vello premultiplies honestly rather
than this adapter double-premultiplying. [`Scene::draw_image`] draws the image
at its intrinsic `width × height`; a per-copy transform scales that to `tile`'s
size and translates it to each copy's origin, composed under the active
`transform`.

peniko 0.6's `ImageData` exposes `x_extend` / `y_extend`, but Vello 0.9's
`Scene::draw_image` draws a single image instance (extend only pads sampling at
the image's own edges, it does not tile across a larger area), so tiling is done
with the SAME bounded per-copy loop as the CPU backend, and both are confined to
`region` by a pushed clip layer. A single tile fully inside `region` with no
tiling takes neither the loop nor the clip — visually identical to the previous
stretch draw. A zero-sized image, tile, or region is skipped.
