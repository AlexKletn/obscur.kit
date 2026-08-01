Shapes the `0` (ZERO, U+0030) glyph through `paragraph` at `font_size` and
returns its advance — the base every `ch` length resolves against (spec §4.5,
CSS Values 4 §6.1.2).

Returns [`None`] when shaping yields no glyph, which is the honest answer
rather than `0.0`: a zero-width `ch` would silently collapse `width: 60ch` to
nothing, while [`None`] routes the caller to
[`LengthContext::ch_base`](obscur_style::property::LengthContext::ch_base)'s specified `0.5em` substitute.

# This measures the same font the text does

Shaped through the same [`measure_face`] + fallback chain + attributes as
[`measure_text_node`], so a `20ch` box and the twenty characters placed in it
are measured against one font, not two. That also means `ch` inherits this
pass's standing limitation — though it is narrower than it reads: the node's
families are passed here and the resolver's fallback chain picks the face, so
`ch` **does** follow `font-family` (a monospace node measures its own `0`).
Only the [`measure_face`] carrier is ignored. What is genuinely open is a
family that resolves to no registered face: this answers `None`, layout takes
`ch_base`'s 0.5em substitute, and paint reads `0` — the two disagree, which is
the seam `ChMetrics` exists to close and does not close here.
