Maps a screen-space point into `node`'s own untransformed coordinate
space.

For the untransformed node — nearly all of them — this is the identity
and returns `point` unchanged. For a transformed one it is the inverse of
the accumulated chain, which is what makes the narrow-phase tests below
(the border box, its corner radii, a canvas's declared region) meaningful
again: those shapes are all expressed in the node's untransformed space,
so the point has to come back to them rather than the shapes going out to
it.
