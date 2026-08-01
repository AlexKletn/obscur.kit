Splits a `:nth-child()` / `:nth-last-child()` argument into its `An+B` part
and the optional `of S` selector list (selectors-4 §13.3.1:
`:nth-child( <an+b> [of <complex-real-selector-list>]? )`).

The split is made at the first `of` **keyword** — an `of` that stands alone as
an identifier at bracket depth zero — so the `of` inside `-n+3 of .of-item`
or inside `[data-of]` is not mistaken for the separator. Returns the whole
argument and [`None`] when there is no such keyword.
