A small authored stylesheet that exercises several CSS features at once:
a `:root` custom property, a flex container, a `var()` reference, a
`:hover`-gated declaration, an `@keyframes` animation, and — on purpose —
two things the frontend does not support (`@import` and `float`) so we can
show they are reported as diagnostics instead of aborting the compile.
