Folds a `filter` op list's **color** ops (CSS Filter Effects §) into a single
4×5 affine color matrix, applied in author order, or [`None`] when the list has
no color op (so the coordinator emits **no** `ColorMatrix` effect — byte
identical to no filter).

Every color function is an affine transform `rgba' = M·rgba + bias`, so a chain
composes by matrix multiply: applying `(Mₙ, bₙ)` after the running `(M, b)`
gives `M' = Mₙ·M`, `b' = Mₙ·b + bₙ`. Starting from the identity and folding
left-to-right yields one matrix equivalent to the whole chain. The result is
flattened row-major: `arr[r*4+c]` is the 4×4 linear entry, `arr[16+r]` the bias.
