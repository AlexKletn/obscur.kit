A 2D affine decomposed into its animatable components (CSS Transforms
§14.5.1).

This is the intermediate representation transforms are interpolated through:
linear factors (`translate`, `scale`, `skew`) that lerp directly and a
rotation `angle` that must be blended along the shortest arc. Recomposing in
the order **translate · rotate · skew · scale** ([`recompose`]) inverts
[`decompose`] exactly, so `decompose(recompose(d)) ≈ d` and
`recompose(decompose(m)) ≈ m` round-trip within epsilon.
