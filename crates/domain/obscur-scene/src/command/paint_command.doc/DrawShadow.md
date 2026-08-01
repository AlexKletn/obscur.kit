Paint a CSS `box-shadow` (spec §5.2). A LEAF command (no nesting effect).

The shadow is a gaussian-blurred rounded rectangle derived from the box's
border box `rect` and corner `radii`: it is translated by `offset`,
inflated (a **drop** shadow, `inset = false`) or deflated (an **inset**
shadow, `inset = true`) by `spread`, and blurred by `blur` (the CSS blur
*radius*, which each renderer maps to a gaussian standard deviation).

The coordinator emits a drop shadow **before** (behind) the box's
background and an inset shadow **after** it; the renderer decides the
geometry from these fields so both backends consume identical parameters
and only rasterization differs. `color` is already resolved (the CSS
`currentColor` default having been substituted by the coordinator).
