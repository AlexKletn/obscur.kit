Lowers one `<shadow>` = `<offset-x> <offset-y> <blur>? <spread>? <color>? &&
inset?` into a [`BoxShadow`] (spec §5.2).

The two required lengths are the `x` / `y` offsets, an optional third and
fourth length are the blur radius and spread, the optional color defaults to
`currentColor` (kept as [`None`], resolved at paint time), and the optional
`inset` keyword may appear in any position. Lengths are `<length>`s (a unitless
`0` is accepted as `0px`; percentages are not valid). Returns [`None`] for a
duplicate color / `inset`, an unparseable component, or fewer than two / more
than four lengths.
