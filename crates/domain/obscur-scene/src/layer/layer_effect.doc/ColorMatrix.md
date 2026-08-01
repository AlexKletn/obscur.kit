Applies a 4×5 affine color matrix to the layer's composited RGBA content
(`rgba' = M · rgba + bias`, per-channel, then clamped to `0..1`).

The 20 entries are the CSS Filter Effects color-matrix folded from an
element's color-op `filter` chain (`grayscale/brightness/contrast/
saturate/sepia/invert/hue-rotate/opacity`): a 4×4 linear part in **row
major** order (`m[row][col] == matrix[row * 4 + col]`, indices `0..16`)
followed by the 4-entry bias (`bias[row] == matrix[16 + row]`). The
coordinator composes the whole affine color-op chain into this single
matrix; the identity chain emits **no** effect (byte-identical no-filter).
