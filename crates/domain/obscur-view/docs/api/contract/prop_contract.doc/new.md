Creates a prop contract for `name` with declared value type
`value_type`.

The prop starts out optional and without a default; use
[`PropContract::required`] and [`PropContract::with_default`] to refine
it in a builder style.
