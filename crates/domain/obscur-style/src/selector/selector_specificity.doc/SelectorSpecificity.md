The standard CSS specificity triple used to resolve cascade conflicts.

A selector's specificity is the ordered triple `(a, b, c)` where:

- `a` counts ID selectors (e.g. `#main`),
- `b` counts class selectors, attribute selectors, and pseudo-classes
  (e.g. `.item`, `[type="text"]`, `:hover`),
- `c` counts type selectors and pseudo-elements (e.g. `div`, `::before`).

Specificity is compared **lexicographically**: `a` dominates, then `b`,
then `c`. This type derives [`Ord`] so that ordering is exactly CSS
specificity ordering — a single ID selector `(1, 0, 0)` outranks any number
of class selectors `(0, N, 0)`. When two declarations have equal
specificity, the cascade falls back to source order (handled elsewhere).

The concrete selector syntax and the matching engine live in `obscur-css`;
this crate only models the typed specificity value the cascade consumes.
