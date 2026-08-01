Lowers an `object-fit` value into a [`PropertyValue::ObjectFit`] (CSS Images
§5.5).

Accepts a single keyword: `fill` (the initial) / `contain` / `cover` / `none` /
`scale-down`. Any other syntax yields [`None`] (the declaration degrades to an
unsupported-feature diagnostic and the initial `fill` applies).
