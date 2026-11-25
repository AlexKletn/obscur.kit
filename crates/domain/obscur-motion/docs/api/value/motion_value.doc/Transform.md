A 2D `transform`: the six affine coefficients `[a, b, c, d, e, f]` of a
`matrix(a, b, c, d, e, f)` (the lowered `<transform-function>+` list).

Unlike the other variants this does **not** blend component-wise: it
interpolates by matrix decomposition (CSS Transforms §14), so a
`rotate(0°) → rotate(90°)` transition rotates through `rotate(45°)`
instead of shearing and shrinking the box. See
[`interpolate_checked`](Self::interpolate_checked).
