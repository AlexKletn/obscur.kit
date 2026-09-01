Parses `tokens` — the text a custom property was declared with, the name and
the colon already stripped — as a value of `property`.

Returns [`None`] when the text matches no value `property` accepts, which is
CSS Variables 1 §3's *invalid at computed-value time*; the caller spells that
`unset`. Answering [`Some`] with a value the property cannot carry is not a
way to report failure — the resolver judges the shape afterwards and would
spell that `unset` too, having done the work twice.

An implementor is called during resolution, once per substitution that reaches
a raw token sequence, so it must be cheap enough to run per property per node.
