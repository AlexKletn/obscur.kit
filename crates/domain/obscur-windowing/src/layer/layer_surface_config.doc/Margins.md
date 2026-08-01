Per-edge gaps between a layer surface and the edges it is anchored to.

Separate from [`Insets`](obscur_geometry::Insets) because these are **integer
logical pixels** in the protocol, and rounding a float at the boundary is a
decision better made once, here, than at every call site.
