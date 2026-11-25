Resolves this length to concrete logical pixels against `ctx`.

* `Px` returns its value directly; the font- and viewport-relative units
  multiply their coefficient by the matching base from `ctx` — `Ch`
  against [`LengthContext::ch_base`], which substitutes `0.5em` when the
  `0` glyph was not measurable.
* `Percent` resolves only when `ctx.percent_basis` is known (it usually
  is not here — bare percentages are resolved later by layout against the
  containing block), otherwise it returns [`None`].
* `Auto` has no numeric value and returns [`None`].
