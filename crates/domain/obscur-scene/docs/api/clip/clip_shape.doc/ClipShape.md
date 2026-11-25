The geometric region a clip confines subsequent drawing to (spec §5.8).

A `ClipShape` is pure, backend-independent geometry: a plain rectangle, a
rounded rectangle, or an arbitrary [`Path`]. It carries no rendering state
and references no GPU or renderer types. Rectangular and rounded-rectangular
shapes are cheap to apply; a path shape is more general but still describes
only geometry.
