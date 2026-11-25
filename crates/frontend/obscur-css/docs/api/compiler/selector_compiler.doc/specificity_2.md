Computes the CSS `(a, b, c)` specificity of a complex selector (§4.14).

`a` counts id selectors, `b` counts class/attribute/pseudo-class
selectors, and `c` counts type/pseudo-element selectors; the universal
selector contributes nothing. Components are summed (saturating) across
every compound in the complex selector.

This delegates to the lowered `obscur-style` model rather than counting
the parsed AST a second time, so the flat-cascade path and the
runtime-matcher path cannot disagree. That matters for the selectors-4
§15 "evaluation context" pseudo-classes (`:is()`, `:not()`, `:where()`,
`:has()`, `:nth-child(… of S)`), whose specificity is a function of their
*argument* — which only exists once the raw argument text has been
re-parsed by [`lower_complex`](Self::lower_complex).
