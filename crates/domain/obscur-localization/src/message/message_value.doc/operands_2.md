Returns the operands a `plural` arm selects on, for a
[`Number`](Self::Number); `None` for anything else.

A `plural` selector over a non-number has no answer, and this returning
`None` is how the formatter finds that out rather than inventing one.
