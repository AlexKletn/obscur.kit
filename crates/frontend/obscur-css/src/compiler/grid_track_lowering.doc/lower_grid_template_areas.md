Lowers a `grid-template-areas` value into a [`PropertyValue::GridAreas`]
(spec §4.2, the named-areas subset).

The value is a sequence of quoted strings, one per grid row; each string is a
whitespace-separated list of cell tokens, where a token is a `<custom-ident>`
area name or a run of one-or-more `.` characters denoting a null (empty) cell.
Every row must have the same number of cells, and every named area must form a
filled rectangle — both are enforced by
[`GridAreaTemplate::from_rows`](GridAreaTemplate), which returns [`None`] on a
ragged grid or a non-rectangular area, degrading the declaration to an
unsupported-feature diagnostic rather than laying out against a malformed grid.
A value that is not a list of strings (or an empty list) is likewise [`None`].
