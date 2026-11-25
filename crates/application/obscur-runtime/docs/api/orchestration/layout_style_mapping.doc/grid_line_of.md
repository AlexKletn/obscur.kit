Reads one grid-line longhand and converts the style-local [`GridLineValue`] into
an `obscur_layout` [`GridLine`], defaulting to [`GridLine::Auto`] when the
property is absent or is not a grid-line value.

A [`GridLineValue::Named`] line cannot be resolved on the item alone (its name
is defined on the container's template), so it maps to [`GridLine::Auto`] here
and is patched to a concrete line afterwards by
[`resolve_named_grid_lines`](super::resolve_named_grid_lines), mirroring how
`grid-area: <name>` is resolved against the parent's `grid-template-areas`.
