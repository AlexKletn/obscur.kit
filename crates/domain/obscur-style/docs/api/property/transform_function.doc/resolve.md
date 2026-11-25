The 2D affine `[a, b, c, d, e, f]` this function contributes, or [`None`] when a
distance is still unknown (a `%`, or a `ch` with no measured advance).

[`None`] is the whole point of the type: a caller that cannot resolve the
function must decline to build a matrix, never substitute `0`.
