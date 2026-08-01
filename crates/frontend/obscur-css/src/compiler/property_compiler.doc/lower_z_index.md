Lowers a `z-index` value: an `<integer>` or the `auto` keyword (spec §5.1).

A bare integer is taken verbatim as a [`PropertyValue::Number`] (negative
values allowed, e.g. `z-index: -1`); a non-integral number is not a valid
`z-index` and yields [`None`]. The `auto` keyword lowers to the reserved
[`layout_keyword::Z_INDEX_AUTO`] handle so the runtime reads one keyword
identity across crates. The CSS-wide keywords (`inherit` / `initial` /
`unset`) still take priority. Any other value yields [`None`] so the
declaration degrades to an unsupported-feature diagnostic.
