Reserved symbol for the `test-id` attribute — a node's stable testing
identifier (spec §7.5).

# Deliberately not a style attribute

Spec §7.5 requires that these identifiers *do not affect layout, do not
affect CSS specificity, and do not affect visual output*. That is enforced
here by construction rather than by convention: `test-id` is **not** part of
[`StyleAttributes`](super::StyleAttributes), so
[`extract`](super::StyleAttributes::extract) never sees it and the selector
matcher has nothing to match on. It lives only in the node's retained
property list, which is where a resolver reads it.

It occupies a namespace of its own (`0x0000_02xx`), distinct from both the
CSS-relevant reserved attributes (`0x0000_000x`) and the reserved events
(`0x0000_010x`), so the separation is visible in the handle itself.

Being an ordinary property also means it satisfies §7.5's "may be compiled
out of production builds": a build that does not author it simply carries no
such property, and costs nothing.
