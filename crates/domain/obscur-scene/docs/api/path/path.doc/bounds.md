The axis-aligned bounding box of every point referenced by the path, or
`None` if the path is empty (has no points).

Control points of quadratic and cubic segments are included, so the box
is a conservative bound: it always contains the drawn curves but may be
larger than their tight bounds. Never panics.
