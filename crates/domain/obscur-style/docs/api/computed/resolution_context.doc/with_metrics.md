Attaches the port that measures the `0` glyph, making `ch` absolutize like
every other relative unit.

Without one, `ch` cannot be folded to pixels here and is left symbolic on
the [`ComputedStyle`](super::ComputedStyle) for a downstream consumer that can measure it — which
is what the resolver did for every caller before this port existed. That
path is still correct for a consumer that measures (layout does), and still
wrong for one that does not (paint reads `0`), so a caller that *can* supply
metrics should.
