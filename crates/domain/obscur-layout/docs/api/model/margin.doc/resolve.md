This edge as a plain length, against a containing-block inline size of
`basis`.

* `Px` is already a length.
* `Percent` is `basis × percent / 100`. An `basis` that is not finite —
  which is what a `max-content` probe offers — yields `0`: CSS Sizing 3
  §5.2 treats a percentage margin as zero when computing an intrinsic
  contribution, for the same reason it does for a percentage size (there is
  nothing definite to be a percentage *of*). Passing `0.0` as the basis is
  the honest way for a caller with no containing block to say so.
* `Auto` is `0` — its used value whenever there is no leftover space to
  absorb, which CSS 2.1 §10.3.3 makes the common case: a `width: auto` block
  already fills its container.

Use this wherever an edge is *consumed* as a length; use
[`is_auto`](Self::is_auto) where the leftover actually gets distributed,
because this collapses that question to `0`.
