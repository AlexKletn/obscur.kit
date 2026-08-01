Resolves the four margin longhands into [`MarginEdges`], keeping `auto`
**unresolved** (CSS 2.1 §10.3.3).

This is the margin twin of [`insets_of`], and differs in the one way that
matters: neither `auto` nor a percentage is folded to a number here, because
neither *has* one yet.

* `auto` means "take the leftover", and how much is left depends on the box's
  used width, which layout has not computed. That premature `0` is exactly what
  used to make `margin: 0 auto` do nothing.
* A percentage resolves against the *containing block's* inline size (CSS 2.1
  §8.3 — the width, on all four edges), which this node's own
  [`LengthContext`] does not know: `ctx.percent_basis` is the node's own
  basis, not its parent's content width. Layout resolves it where the
  containing block is in hand.

`calc()` mixing a percentage still folds to `0` (the expression cannot be
resolved without a basis), as it does for every other length here.
