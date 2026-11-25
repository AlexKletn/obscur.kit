Constructs a context with an unknown percentage basis and an unmeasured
`ch`.

`em` / `rem` and the viewport-relative units resolve; a `%` term does not
(it is resolved later, by layout, against the containing block), and `ch`
falls back to [`ch_base`](Self::ch_base)'s substitute until a caller that
can measure the `0` glyph supplies it via
[`with_ch_width`](Self::with_ch_width).
