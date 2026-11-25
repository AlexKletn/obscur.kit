Records a color-matrix pass sampling `source` and writing the transformed
result into `destination` (replace), applying the folded 4×5 `matrix`.

`matrix` is the flattened 20 entries the shader's uniform expects verbatim:
the four rows of the linear 4×4 part (`matrix[r*4 + c]`) followed by the
4-entry bias (`matrix[16 + r]`) — exactly five `vec4`s.
