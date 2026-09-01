Parses `tokens` as a value of `property`.

This is the CSS grammar reaching the computed-value step: a custom property
kept the text it was declared with, and `property` is what finally says which
grammar to read it against — a family list for `font-family`, a shadow list
for `box-shadow`, a `<color>` for every colour longhand.

Returns [`None`] when the text is not a value `property` accepts, which the
resolver spells `unset` (CSS Variables 1 §3, *invalid at computed-value
time*).

The grammar is not restated here: this delegates to the same
`lower_for_property` a literal declaration goes through, so `--o: hidden;
overflow: var(--o)` computes the `hidden` the literal computes, and `--o:
sideways` is the [`None`] the literal `overflow: sideways` is. Holding a
second copy of that rule here is exactly how the two spellings came to
disagree once.

The interner this passes is private to the call and dropped on return, so an
*open*-set keyword substituted this way gets a symbol no stylesheet shares —
the same condition every per-sheet interner already creates, and harmless
because nothing reads those symbols as values.
