Resolves this dimension against a `base` length, in logical pixels.

* [`Dimension::Px`] resolves to its stored value.
* [`Dimension::Percent`] resolves to `base * percent / 100.0` — but only
  against a **definite** base; see below.
* [`Dimension::Auto`] has no resolvable value and yields [`None`].
* The intrinsic keywords yield [`None`]: they are not resolvable by
  arithmetic at all, because the answer is a property of the content. An
  algorithm that supports them matches on them *before* calling this and
  measures; one that does not falls back to [`None`] and so treats them as
  `auto` — see [`is_intrinsic`](Self::is_intrinsic).

# An indefinite base makes a percentage `auto`

`base` is not finite while content is being measured under unbounded space
(a `max-content` probe offers an infinite inline axis, and a `height: auto`
block offers an infinite block axis). CSS Sizing 3 §5.1: a percentage
against an indefinite basis "behaves as `auto`". Resolving it arithmetically
instead would return infinity and poison every size derived from it — a
`max-content` box containing a `50%` child would come out infinitely wide.
