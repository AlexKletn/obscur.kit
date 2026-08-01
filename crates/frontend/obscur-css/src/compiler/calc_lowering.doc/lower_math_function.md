Lowers a math function (`calc` / `min` / `max` / `clamp`, or a bare
parenthesised group with an empty name) into a [`LengthExpr`].

Returns [`None`] for any other function name, or when the argument list is
not a well-formed length expression (a stray operator, a `*`/`/` with no
numeric operand, an empty `min()`/`max()`, or a `clamp()` without exactly
three arguments).

A `clamp()` bound may also be the keyword `none` — see
[`parse_clamp_bound`].
