Decomposes a 2D affine into translation, scale, shear and rotation
(CSS Transforms §14.5.1 "Decomposing a 2D matrix"; Graphics Gems `unmatrix`).

Given `[a, b, c, d, e, f]`:
1. `translate = (e, f)`.
2. `scaleX = hypot(a, b)`; normalize column 0 `(a, b) /= scaleX`.
3. `skew = a·c + b·d` (XY shear); make column 1 orthogonal to column 0:
   `c -= a·skew; d -= b·skew`.
4. `scaleY = hypot(c, d)`; normalize column 1 `(c, d) /= scaleY`;
   `skew /= scaleY`.
5. If the (normalized) determinant `a·d - b·c < 0` the affine includes a
   reflection: negate `scaleX` and column 0 `(a, b)` so the remaining basis
   is a pure rotation.
6. `angle = atan2(b, a)`.

Degenerate (collapsed) axes — `scaleX` or `scaleY` at or below
[`DEGENERATE_SCALE`] — are guarded: normalization substitutes a unit basis
rather than dividing by ~zero, keeping the result finite.
