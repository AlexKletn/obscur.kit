The `pointer-events: auto` value (the initial): the element takes part in hit
testing as it always did — a pointer over its box lands on it.

On an element inside a `pointer-events: none` subtree this is the reset: the
property inherits, so `auto` on a descendant overrides the ancestor's `none`
and makes that descendant hittable again while its ancestor stays
transparent.
