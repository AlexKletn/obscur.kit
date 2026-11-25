The typed source of color used to paint a shape (spec §5.1).

A `Brush` is one of the concrete paint sources: a flat [`SolidBrush`] or one
of the gradient brushes. It carries only backend-independent data; a
renderer interprets it into device pixels.

Image and pattern brushes are a later addition.
