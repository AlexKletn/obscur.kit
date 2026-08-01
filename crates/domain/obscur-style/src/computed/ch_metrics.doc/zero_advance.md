The advance of the `0` (ZERO, U+0030) glyph for the font `font` selects,
in logical pixels — the base every `ch` length resolves against.

`font` is guaranteed to carry the node's already-resolved `font-size`
(absolute px) plus its `font-family` / `font-weight` / `font-style`, which
is everything needed to select a face. It carries nothing else that can be
relied on: the resolver calls this *mid-resolution*, before the remaining
properties are computed, precisely because their `ch` terms depend on the
answer. An implementor that reads any other property is reading a value
that is not there yet.

Returns [`None`] when the advance cannot be measured — no face, or a face
with no `0`. [`None`] is the honest answer and is **not** the same as
`Some(0.0)`: it routes the caller to
[`LengthContext::ch_base`](crate::property::LengthContext::ch_base)'s
specified `0.5em` substitute (§6.1.2: *"in the cases where it is impossible
or impractical to determine the measure of the `0` glyph, it must be
assumed to be 0.5em wide"*), whereas `Some(0.0)` would silently collapse
`width: 60ch` to nothing.

That distinction holds **at layout only**. Paint leaves an unmeasured `Ch`
symbolic and reads it as 0, so there `None` and `Some(0.0)` are
indistinguishable — which is the seam this port narrows rather than closes:
a family that resolves to no face still lays out against the 0.5em
substitute and paints at 0.
