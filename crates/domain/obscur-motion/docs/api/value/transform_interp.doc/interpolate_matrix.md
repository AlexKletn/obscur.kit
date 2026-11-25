Interpolates two 2D affines at progress `t` by decompose → interpolate →
recompose (CSS Transforms §14).

This is the transform-aware blend the motion engine uses instead of a naive
component-wise matrix lerp: e.g. `rotate(0°)` → `rotate(90°)` at `t = 0.5`
yields `rotate(45°)` with scale preserved (`hypot(a, b) == 1`), whereas a raw
lerp of `[1,0,0,1]`/`[0,1,−1,0]` gives `[0.5,0.5,−0.5,0.5]` — a spurious
`hypot(0.5, 0.5) ≈ 0.707` shrink and shear.

Interpolating a matrix with itself at any `t` returns it within epsilon
(decompose then recompose is identity), and `identity → identity` stays
identity.
