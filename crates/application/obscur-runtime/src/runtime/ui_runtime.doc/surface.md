The host surface frames are laid out and painted into: its viewport, its
device scale, and the two offsets that place the document inside it.

One field rather than the four it replaces, because they are one coordinate
system — see [`SurfaceGeometry`], which keeps the render transform and the
pointer mapping exact inverses of each other.
