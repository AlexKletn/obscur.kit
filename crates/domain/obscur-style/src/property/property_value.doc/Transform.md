A resolved CSS 2D `transform`: the authored `<transform-function>+` list
composed left-to-right into a single 2x3 affine (see [`Transform`]).
`transform: none` lowers to an absent value, not this. The scene
coordinator wraps the affine about the element's `transform-origin` and
brackets the subtree's paint with it (spec §5.2).
