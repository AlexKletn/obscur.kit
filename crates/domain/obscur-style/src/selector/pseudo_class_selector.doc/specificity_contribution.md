Returns the specificity this pseudo-class contributes (selectors-4 §15).

Most pseudo-classes count as one class-level selector, `(0, 1, 0)`. The
four that "provide evaluation contexts for other selectors" are defined
specially, and they genuinely differ:

- `:is()`, `:not()` and `:has()` — "The specificity of an `:is()`,
  `:not()`, or `:has()` pseudo-class is replaced by the specificity of the
  most specific complex selector in its selector list argument." The
  pseudo-class's own `(0, 1, 0)` is **replaced**, not added to, so
  `:is(em, #foo)` is `(1,0,0)` and `:not(em, strong#foo)` is `(1,0,1)`.
- `:where()` — "The specificity of a `:where()` pseudo-class is replaced
  by zero"; neither it nor its arguments contribute.
- `:nth-child(An+B of S)` — "Analogously, the specificity of an
  `:nth-child()` or `:nth-last-child()` selector is the specificity of the
  pseudo-class itself (counting as one pseudo-class selector) **plus** the
  specificity of the most specific complex selector in its selector list
  argument (if any)." Here the base *is* added, which is why
  `S:nth-child(An+B)` and `:nth-child(An+B of S)` tie.

An empty argument list has no most-specific member and so contributes
nothing, leaving `:is()`/`:not()`/`:has()` at zero and
`:nth-child(… of )` at the bare `(0, 1, 0)`.
