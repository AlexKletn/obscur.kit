2D transform interpolation via matrix decomposition (CSS Transforms Level 1
§14).

`transform`s are authored as a `<transform-function>+` list but reach this
crate already lowered into a single 2D affine `[a, b, c, d, e, f]` (the six
coefficients of `matrix(a, b, c, d, e, f)`). Because the primitive list is
not retained, the per-function "matching transform lists" fast path of
[CSS Transforms §14.4] is unavailable, so this module implements the general
**matrix** fallback: interpolate two opaque affines by
**decompose → interpolate → recompose**.

* [`decompose`] implements [§14.5.1 "Decomposing a 2D matrix"][decomp] (the
  Graphics Gems `unmatrix` of Spencer W. Thomas): peel `translate`, `scale`,
  the XY `skew` (shear) and the rotation `angle` off the affine.
* [`interpolate`] blends two decompositions per [§14.4]: `translate`,
  `scale` and `skew` lerp linearly while `angle` takes the **shortest arc**
  (never the long way around).
* [`recompose`] rebuilds an affine in the spec order
  **translate · rotate · skew · scale**, the exact inverse of [`decompose`]
  so the pair round-trips within epsilon.

Only the **2D** path is implemented: the engine's `transform` is a 2D
`[f32; 6]` with no 4×4 matrix type, so the 3D `unmatrix` + quaternion Slerp
of [§14.5.2] is out of scope (a follow-up gated on a 4×4/quaternion type).

[CSS Transforms §14.4]: https://www.w3.org/TR/css-transforms-1/#interpolation-of-transforms
[§14.4]: https://www.w3.org/TR/css-transforms-1/#interpolation-of-transforms
[decomp]: https://www.w3.org/TR/css-transforms-1/#decomposing-a-2d-matrix
[§14.5.1]: https://www.w3.org/TR/css-transforms-1/#decomposing-a-2d-matrix
[§14.5.2]: https://www.w3.org/TR/css-transforms-1/#decomposing-a-3d-matrix
