Whether the surface stretches horizontally, because it is anchored to
both the left and right edges.

When this holds, the width in [`LayerSurfaceConfig`](super::LayerSurfaceConfig)
is not what the surface gets — the compositor decides it. Reading the
configure event, rather than the request, is the only correct way to know
the real size.
