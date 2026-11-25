Lowers a `transition` shorthand declaration into one template per
comma-separated layer.

A layer without a resolvable [`PropertyId`] is skipped (for example a
`transition: all …` layer, whose property cannot be enumerated).
