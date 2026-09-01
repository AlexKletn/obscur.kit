The properties whose value is a bare `<number>`.

A unitless zero is not the exception it looks like: `0` is a valid length for
every length-valued longhand (CSS Values §4.5.3), and the grammar lowers it to
`0px` before it reaches here, so `width: var(--zero)` is `0px` and not the
`auto` an `unset` would have left. A number that arrives on a length property
is therefore a genuinely unitless one — `width: 2` — which no length property
carries. The two properties that take both shapes, `line-height` and
`tab-size`, are listed here as well as under `takes_length`.
