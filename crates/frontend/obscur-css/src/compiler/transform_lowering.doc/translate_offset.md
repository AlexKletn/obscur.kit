Reads one `translate` distance as a [`LengthExpr`]: a unitless `0`, any absolute
or font/viewport-relative `<length>`, or a math function (`calc`, `min`, `max`,
`clamp`) over them.

A **percentage** — bare, or inside the math function — yields [`None`], which
makes the whole declaration an `UnsupportedCssFeature`. That is deliberate and
is the point of this function: a `%` here means the element's own border box,
which nothing before layout knows, and the previous answer (`0`) turned
`translate(-50%, -50%)` into a silent no-op that looked like a stylesheet typo.
