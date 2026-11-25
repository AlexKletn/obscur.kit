Folds a specified `transform` function list into the used 2D affine matrix.

This is where `transform: translate(2em, 10vw)` becomes a matrix. The CSS
compiler cannot do it — `em` is this node's font size and `vw` is the viewport,
neither of which exists at lower time — so the list arrives carrying its
[`LengthExpr`](crate::property::LengthExpr)s and is composed here, in author
order, against the same [`LengthContext`](crate::property::LengthContext) that
folds a `box-shadow` offset.

If ANY distance is still unknown the value stays a list. Composing the rest
would be worse than declining: a `translate` dropped from the middle of a chain
places the element at a distance nobody wrote, and the element would look like
the declaration had simply been ignored. What this context *could* fold is kept,
so a later pass — a `ch` that has since been measured — can finish the job.
