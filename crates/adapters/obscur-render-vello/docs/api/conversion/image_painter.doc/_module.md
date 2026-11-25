Draws a decoded [`SceneImage`] onto a [`vello::Scene`], tiled and clipped like
a CSS background (spec §5.16, CSS Backgrounds §3).

One `DrawImage` command is a *background layer*, not a single blit: the tile
rect says where one copy goes, the region says how far the layer may paint, and
the two `repeat` flags say whether copies march out along each axis. Resolving
that into copies — the index range per axis, the bounded-loop guard, the region
clip, and the zero-copy `SharedPixels` → peniko [`Blob`] wrapping — is this
file's vocabulary; [`draw_image`] is the single entry point the command walk
calls.
