Converts an obscur-geometry [`AffineTransform`] into a
[`tiny_skia::Transform`]. The obscur coefficients `[a, b, c, d, tx, ty]` map to
tiny-skia's row order `(sx, ky, kx, sy, tx, ty)` one-to-one (both map a point
`(x, y)` to `(a·x + c·y + tx, b·x + d·y + ty)`).
