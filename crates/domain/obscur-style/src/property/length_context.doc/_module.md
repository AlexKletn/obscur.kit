The resolution context a relative length is evaluated against.

Font-relative units (`em` / `rem` / `ch`), viewport units (`vw` / `vh` /
`vmin` / `vmax`) and `calc()` / `min()` / `max()` / `clamp()` expressions carry no
absolute value on their own: they only become a concrete number once a
[`LengthContext`] supplies the bases they resolve against (spec §4.5).

This context serves both of the folds that need those bases:

* the **computed-value** fold
  ([`Length::absolutize`](crate::property::Length::absolutize)), which runs
  once per node and retires `em` / `rem` / `vw` / `vh` for good — after it,
  no consumer of a computed style needs a context to read a length; and
* the **use-time** fold
  ([`Length::resolve_px`](crate::property::Length::resolve_px) /
  [`LengthExpr::resolve`](crate::property::LengthExpr::resolve)), which layout
  still runs for the terms the first fold cannot absolutize — a `%` (it needs
  [`percent_basis`](LengthContext::percent_basis), which only layout knows)
  and a `ch` (it needs [`ch_width`](LengthContext::ch_width), which only a
  caller with a shaper can measure).
