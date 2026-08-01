These edges as plain [`Insets`], against a containing-block inline size of
`basis`.

`basis` resolves percentage edges — all four of them, since CSS 2.1 §8.3
resolves even `margin-top` against the containing block's *width*. Every
`auto` reads as `0`.

The lossy direction, and deliberately so: it is for the callers that consume
margins purely as lengths and have no leftover space to hand out. A caller
that *distributes* leftover space must read the edges directly and ask
[`Margin::is_auto`] — collapsing to `0` first is exactly how `auto` gets
lost. See [`Margin::resolve`] for what a non-finite or zero `basis` means.
