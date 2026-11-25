What a `var()` substitution computes to when what came back does not fit the
property it landed on: [`substituted_value`].

CSS Variables 1 §3 gives this one answer and no other. A substitution that is
*invalid at computed-value time* makes the property compute to `unset`, which
css-cascade-5 §7.3.3 spells out as: the inherited value where the property
inherits, the initial value where it does not.

Absence is a third behaviour the specification does not offer, and it is the
one a wrong-typed value falls into by accident: `font-family:
var(--font-body)` where `--font-body` holds a length reaches the longhand,
computes to a length, and reads as no family at all — rather than as the
family the parent set.

The judgement belongs here rather than in the CSS compiler because it cannot
be made any earlier: what a reference stands for does not exist until the
cascade has run, so the compiler carries the reference and this step decides.
For the same reason this is where a custom property's declared text is finally
parsed — a token sequence has no grammar until it lands on a property — which
this step asks for through
[`CustomValueParser`](super::CustomValueParser) rather than owning.

It applies only to a value that arrived through a reference. An ordinary
declaration was already judged, against the real grammar, by the parser that
lowered it.
