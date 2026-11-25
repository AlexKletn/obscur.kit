An embedded, externally-owned surface placed into the view tree.

`ForeignSurface` is one of the minimal built-in primitives (spec §2.6). It
reserves a region for a surface the framework does not paint — a native
widget, video output, or an external GPU-rendered surface (spec §5.5) —
referenced by an opaque handle key the host resolves to the real surface. It
is a leaf element primitive with no children: it lowers to a
[`ViewNode::Element`] carrying the surface handle as a property.
