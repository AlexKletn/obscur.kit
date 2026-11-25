Folds this length's **absolutizable** units to [`Length::Px`], leaving
every other term symbolic (CSS Values 4 §5: the computed value of a
`<length>` is absolute).

`em` / `rem` / `vw` / `vh` / `vmin` / `vmax` all resolve from numbers a
computed-value pass already holds, so they fold here and no downstream
consumer needs a [`LengthContext`] to read them. Three terms do not fold:

* `Percent` resolves against a containing block that layout has not
  produced yet, so it stays a percentage. (`ctx.percent_basis` is ignored
  deliberately: a caller that knows the basis — `font-size: 150%` against
  the parent's size — folds it with [`resolve_px`](Self::resolve_px)
  instead, where the basis is a stated input rather than a silent one.)
* `Auto` has no numeric value at all.
* `Ch` folds **only** when `ctx` carries a measured `ch_width`. Without
  one, folding it here would bake in [`LengthContext::ch_base`]'s `0.5em`
  substitute and permanently mis-size the monospace column `ch` exists
  for; leaving it symbolic routes it to a consumer that *can* shape the
  `0` glyph.
