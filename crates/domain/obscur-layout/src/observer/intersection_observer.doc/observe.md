Records the current `target` and `root` rectangles, returning an
[`IntersectionChange`] when the target crosses the threshold (or on the
first observation).

The ratio is the overlap area divided by the target area (`0.0` when the
target has zero area). The target is intersecting when
`ratio >= threshold`. Returns `None` when the intersecting state is
unchanged.
