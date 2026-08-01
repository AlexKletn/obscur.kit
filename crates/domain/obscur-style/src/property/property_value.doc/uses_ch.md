Returns `true` when resolving this value needs the `0` glyph measured —
that is, when it contains a `ch` term anywhere.

# Why a caller asks first

Every other unit folds from numbers the caller already has. `ch` alone
needs text *shaped* to answer, which costs a font-fallback chain and a
shaping call. Since almost no stylesheet writes `ch`, a caller checks this
first and measures only on a hit; the overwhelmingly common answer is
`false`, and this scan allocates nothing to produce it.

# This must mirror `absolutize_value`

This predicate exists to decide whether the computed-value step should pay
for a measurement before folding a value. It is therefore only correct if
it answers `true` for **every** value that fold could resolve a `ch` in —
a `false` here means the fold runs without a `ch` base, and any `ch` inside
survives as a symbolic term that paint reads as `0`.

So the two walks are a pair: `absolutize_value` visits every [`Length`] a
[`PropertyValue`] carries, and this visits the same ones asking only
[`Length::is_ch`]. Both matches are exhaustive rather than wildcarded, so a
new length-bearing variant cannot quietly default to "no `ch` here" in
either.

This used to answer `false` for [`BoxShadow`](Self::BoxShadow),
[`BackgroundSize`](Self::BackgroundSize) and friends — a documented gap that
made `box-shadow: 1ch 0` resolve to `0` silently. That gap was a
*consequence* of `ch` being unresolvable at computed-value time: widening
the predicate alone would have bought a shaping call whose answer nothing
read. Now that [`ChMetrics`](crate::computed::ChMetrics) threads the
measurement into the fold, the answer is read, and these arms close with it.
