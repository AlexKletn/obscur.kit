Returns the specificity this simple selector contributes.

Per CSS: id → `(1, 0, 0)`; class, attribute, and pseudo-class →
`(0, 1, 0)`; type and pseudo-element → `(0, 0, 1)`; the universal
selector contributes nothing.

Pseudo-classes delegate to
[`PseudoClassSelector::specificity_contribution`], because the four
selectors-4 §15 "evaluation context" forms — `:is()`, `:not()`,
`:where()`, `:has()` — do **not** contribute a flat `(0, 1, 0)`.
