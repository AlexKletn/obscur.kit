Reads `property` off `computed` as a [`PropertyValue::Length`] resolved to
logical pixels against the node's font context (`px` / `em` / `rem`, via
[`length_to_px`]), or `0.0` for an absent property, a non-length value, or a
unit not resolved for spacing (`%` — invalid for letter-/word-spacing — and
the viewport units).
