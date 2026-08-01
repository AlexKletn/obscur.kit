Returns the single precomposed character (as a `String`) a sub-run should
be shaped as under the css-fonts-4 §5.3 canonical-equivalence rule, or
[`None`] when the run shapes as-is.

The rule fires only when **all** of these hold: `sub` is more than one
codepoint; its canonical composition ([NFC](UnicodeNormalization::nfc)) is a
**single** scalar `c` distinct from `sub`; and the resolver selects the same
font for `c` as it did for the whole sequence. That last check preserves the
spec's precedence — a font that *directly* supports the decomposed sequence
(or a different font in the chain) keeps the base + mark glyphs — so
composition happens exactly when the resolved font supports the precomposed
character but not the sequence as authored (the canonical-equivalence branch
of a coverage-driven resolver).
