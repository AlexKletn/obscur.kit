Parses a gradient `at <position>` list (the tokens after `at`) into a
box-relative `(x, y)` center. Reuses the `transform-origin` component grammar:
a one-token form sets the axis its keyword names (the other axis defaults to
`50%`), a two-token `<x> <y>` form accepts the `top left`-style keyword order
(a vertical-first / horizontal-second pair is swapped). Returns [`None`] for an
empty, unparseable, or over-long list.
