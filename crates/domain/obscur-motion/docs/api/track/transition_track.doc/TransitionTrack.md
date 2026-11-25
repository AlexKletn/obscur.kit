A single `from` → `to` interpolation shaped by an easing curve (spec §4.17).

This is the native compile target of a CSS `transition`: one start value, one
end value, and a timing function. Sampling blends `from` toward `to` by the
eased progress. Because the two endpoints must share a [`MotionValue`]
variant to blend, a *type mismatch* (e.g. a scalar transitioning to a point)
cannot interpolate; sampling then falls back to the `to` endpoint rather than
panicking or fabricating a blend.
