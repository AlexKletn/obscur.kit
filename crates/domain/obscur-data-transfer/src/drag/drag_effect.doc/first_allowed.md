Negotiates the effect a drop should carry (spec §6.21 effect
negotiation).

Returns `preferred` when it is present in `allowed` and is not
[`None`](DragEffect::None); otherwise the first non-`None` entry of
`allowed`; otherwise [`None`](DragEffect::None) when `allowed` offers no
acceptable effect.
