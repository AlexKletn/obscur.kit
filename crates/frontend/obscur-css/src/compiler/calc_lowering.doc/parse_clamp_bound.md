Parses one **bound** argument of `clamp()`: either a length expression, or
the keyword `none`. Returns [`None`] when the argument is neither.

css-values-4 §10.2: *"Either the min or max calculations (or even both) can
instead be the keyword `none`, which indicates the value is not clamped from
that side. (That is, `clamp(MIN, VAL, none)` is equivalent to
`max(MIN, VAL)`, `clamp(none, VAL, MAX)` is equivalent to `min(VAL, MAX)`,
and `clamp(none, VAL, none)` is equivalent to just `calc(VAL)`.)"* — so an
unclamped side is lowered to exactly the equivalent the spec names, and
[`LengthExpr::Clamp`] keeps meaning "both sides bounded". `none` is a keyword
in its own right, not an infinity: it is legal **only** as a whole bound
argument, so `clamp(none + 1px, …)` stays invalid.
