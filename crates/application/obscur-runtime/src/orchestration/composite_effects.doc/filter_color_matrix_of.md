The node's `filter` color-op chain folded into ONE 4×5 affine color matrix, or
[`None`] when it authors no color op (the byte-identical no-color path — no
`ColorMatrix` effect is emitted).

See [`color_matrix_of_ops`] for the composition. `blur()` / `drop-shadow()` are
spatial, not color, and are ignored here (they keep their own passes).
