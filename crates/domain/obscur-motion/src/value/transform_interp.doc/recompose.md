Recomposes a 2D affine from its decomposition (CSS Transforms §14, inverse of
[`decompose`]).

Rebuilds the matrix in the spec order **translate · rotate · skew · scale**:
start from the rotation `[cosθ, sinθ, −sinθ, cosθ, 0, 0]`, right-multiply the
shear `[1, 0, skew, 1, 0, 0]` and the scale `(sx, sy)`, then set the
translation `(tx, ty)`. Combining shear·scale into the single 2×2
`[[sx, skew·sy], [0, sy]]` and premultiplying by the rotation yields:

```text
a = sx·cosθ                     c = sy·(skew·cosθ − sinθ)
b = sx·sinθ                     d = sy·(skew·sinθ + cosθ)
e = tx                          f = ty
```

This is the exact algebraic inverse of [`decompose`], so the pair round-trips
within epsilon.
