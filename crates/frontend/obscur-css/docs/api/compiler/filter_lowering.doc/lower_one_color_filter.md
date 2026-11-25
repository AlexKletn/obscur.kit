Lowers one CSS color-matrix `filter` function (`brightness`, `contrast`,
`saturate`, `grayscale`, `sepia`, `invert`, `hue-rotate`, `opacity`) into its
[`FilterOp`], preserving order (CSS Filter Effects §).

The caller has already checked (via [`is_color_filter_name`]) that `name` is a
color function. Every amount accepts a `<number>` or a `<percentage>`
(`100% == 1`); a missing argument defaults to `1` for these functions (and
`0deg` for `hue-rotate`), per spec. `hue-rotate` takes an `<angle>`
(`deg`/`grad`/`rad`/`turn`, or a bare `0`) and is stored in **radians**. Returns
[`None`] (propagated by `?`) when the argument list is malformed, so the author
sees the mistake rather than a silent no-op.
