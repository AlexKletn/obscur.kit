The software backend's blur-free approximation of a CSS `box-shadow` (spec
§5.2, CSS Backgrounds 3 §6.1.1).

This rasterizer owns no gaussian pass, so a shadow cannot be drawn the way the
GPU backend draws it. What it can do exactly is the shadow's *geometry*, and
that geometry is a small algorithm of its own: an outset shadow is the border
box translated by `offset`, inflated by `spread`, with corner radii grown by
`+spread` under §6.1.1's easing; an inset shadow is not an offset blob at all
but a rounded-rect **ring** whose hole contracts by `spread` under the same
rule with the opposite sign. Only after that geometry is settled does the
approximation appear, as a single alpha multiplier standing in for softness.

The narrow interface is one call. The internals are the two §6.1.1 spread
signs, the collapsed-hole degenerate case, and the rule that decides when the
alpha fade applies at all.
