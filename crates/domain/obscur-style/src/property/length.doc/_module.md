CSS length values.

[`Length`] models a single CSS length term: absolute pixels, a percentage
(of some base resolved later during layout), the `auto` keyword, and the
font-relative (`em`, `rem`, `ch`) and viewport-relative (`vw`, `vh`, `vmin`,
`vmax`) units (spec §4.5). The relative units carry no absolute value on
their own; a [`LengthContext`] supplies the bases they fold against.
`calc()` / `min()` / `max()` / `clamp()` combine several such terms and are
modelled by [`LengthExpr`](crate::property::LengthExpr).

# Two folds, for two different jobs

* [`Length::absolutize`] runs **once**, at the computed-value step, and is
  what CSS means by "the computed value of a `<length>` is absolute": it turns
  the font- and viewport-relative units into `Px` and leaves everything that
  genuinely cannot be absolute yet (`%`, `auto`, an unmeasured `ch`) alone. A
  consumer downstream of that step reads `Px` and needs no context.
* [`Length::resolve_px`] folds a length **all the way to a number**, including
  a `%` when the caller states its basis. That is a *use*-time operation: only
  layout knows the containing block a percentage resolves against.
