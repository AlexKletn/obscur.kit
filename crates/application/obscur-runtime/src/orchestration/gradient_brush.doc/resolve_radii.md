Resolves a [`RadialGradient`]'s ending [`shape`](RadialShape) +
[`size`](RadialSize) to a per-axis radius pair `(rx, ry)`, given the gradient's
absolute center `(cx, cy)` and the box `bounds` (CSS Images §3.2).

Let the center's distances to the four sides give `dx_near`/`dx_far` (nearest /
farthest vertical side) and `dy_near`/`dy_far` (nearest / farthest horizontal
side). Then, per the spec's sizing rules:

* **circle** collapses to one radius `r` (`rx == ry`): `closest-side` =
  `min(dx_near, dy_near)`, `farthest-side` = `max(dx_far, dy_far)`,
  `closest-corner` = `hypot(dx_near, dy_near)`, `farthest-corner` =
  `hypot(dx_far, dy_far)`;
* **ellipse** keeps independent axes: `closest-side` = `(dx_near, dy_near)`,
  `farthest-side` = `(dx_far, dy_far)`. For the *corner* extents the ellipse
  keeps the corresponding side ellipse's aspect ratio while passing through the
  corner, which (because that corner sits at exactly the side distances on both
  axes) scales each side radius by `√2`: `closest-corner` =
  `(dx_near, dy_near)·√2`, `farthest-corner` = `(dx_far, dy_far)·√2`.

An explicit length is a concrete radius: `px` directly, a percentage relative to
the corresponding box dimension (spec: radii percentages resolve against the
gradient box's width for `rx` and height for `ry`); for a circle the single
length applies to both axes (a percentage resolves against the box width). Any
other length falls back to `farthest-corner`.
