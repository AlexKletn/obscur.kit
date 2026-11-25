Blitting a decoded [`SceneImage`] as a placed, sized, clipped and tiled
background (spec §5.16, CSS Backgrounds 3 `background-position` /
`background-size` / `background-repeat`).

Unlike the shadow and gradient approximations beside it, this is a *genuine*
sampled bitmap on both backends — which is exactly why it is its own unit: what
it has to get right is not a paint trick but an arithmetic, and that arithmetic
has its own vocabulary. A source buffer in `tiny-skia`'s premultiplied storage,
an intrinsic-to-tile scale, a tile index range over the paint region, and a
bounded loop that cannot be made to spin by a degenerate tile size.

The interface is one call. The internals are the alpha-mode conversion, the
containment test that lets an exact single tile skip building a region mask, the
index range with its one-tile bleed and its [`MAX_TILES_PER_AXIS`] ceiling, and
the per-copy transform.
