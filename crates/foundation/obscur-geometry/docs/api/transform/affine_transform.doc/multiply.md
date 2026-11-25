Composes two transforms: `self.multiply(rhs)` is the transform that
applies `rhs` **first**, then `self` (standard matrix product
`self · rhs`).

When finite coefficients overflow during composition, the product is
recomputed in f64 and narrowed to finite f32 bounds.
