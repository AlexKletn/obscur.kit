Lowers a `grid-auto-flow` value into a single reserved
[`PropertyValue::Keyword`].

`grid-auto-flow` is a *multi-keyword* property: the axis (`row` / `column`)
and an optional `dense` packing keyword may appear together in either order
(`row dense`, `dense column`, …). The CSS-wide keywords (`inherit` /
`initial` / `unset`) take priority when they stand alone. Otherwise the
keyword tokens are joined and folded to a single reserved cross-crate handle
via [`layout_keyword::grid_auto_flow_from_name`] (`row dense` and `column
dense` each collapse to one symbol), so the runtime compares one keyword
identity. A value that is not a valid combination yields [`None`], degrading
to an unsupported-feature diagnostic rather than being dropped.
