A `calc()` / `min()` / `max()` / `clamp()` expression over length terms.

Every node resolves to logical pixels via [`LengthExpr::resolve`]; a term
that cannot fold against the context (for example a `%` with no known basis,
or `auto`) makes the whole expression resolve to [`None`].

Because the leaves carry `f32` data, this derives [`PartialEq`] but not
[`Eq`]/[`Hash`].
