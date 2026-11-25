The node's `filter` colour-op chain folded into ONE 4×5 affine colour matrix,
or [`None`] when it authors no colour op (the byte-identical no-colour path — no
`ColorMatrix` effect is emitted).

Takes the node's already-read op list for the same reason [`filter_blur_of`]
does: one property read, three answers.

See [`color_matrix_of_ops`] for the composition. `blur()` / `drop-shadow()` are
spatial, not colour, and are ignored here (they keep their own passes).
