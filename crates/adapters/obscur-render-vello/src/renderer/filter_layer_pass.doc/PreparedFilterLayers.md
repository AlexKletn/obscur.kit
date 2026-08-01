The [`BlurLayer`]s of one converted scene, each rasterized into its own
margin-padded offscreen and ready to be post-processed and composited back
onto the frame (spec §5.2).

A `filter: blur()` / color-op subtree is not drawn in the main scene: the
converter records it as a [`BlurLayer`], and reproducing it takes **two GPU
steps that cannot share a submission** — Vello rasterizes each sub-scene with
its own `render_to_texture` submits, and only then may the owned gaussian /
color-matrix passes read those textures. That boundary is why this type exists
as a value between the two halves: [`render`](Self::render) performs the first
and holds its results, [`composite`](Self::composite) performs the second.

Both the single-pass path and the segmented `backdrop-filter` path drive it,
which is what keeps a filtered element identical whether or not the same frame
also carries a backdrop.

Each entry pairs the sub-scene's padded offscreen with the absolute pixel
top-left that offscreen composites back at (the layer's bounds moved out by the
gaussian margin), plus the layer descriptor itself.
