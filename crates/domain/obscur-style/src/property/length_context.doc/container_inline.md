The base a `cqi` resolves against, in logical pixels — the small viewport
**inline** size.

The inline axis is horizontal here: this context carries no writing mode,
as `vw`/`vh` do not need one, so `cqi` maps to the width. In a
`vertical-rl` / `vertical-lr` writing mode that is wrong; the axis
mapping is not implemented. See
[`container_width`](Self::container_width) for why the fallback is
always taken.
