Returns this snapshot with `image` attached as its rendered-image stage
(spec §7.4).

This is how the one backend-dependent stage is supplied: a caller
rasterizes the [`scene`](Self::scene) with a concrete renderer (e.g.
`obscur-render-cpu`), builds an [`ImageSnapshot`] from the resulting
pixels, and attaches it here.
