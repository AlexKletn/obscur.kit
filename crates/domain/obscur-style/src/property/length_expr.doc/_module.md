A symbolic length expression: [`LengthExpr`].

`calc()`, `min()`, `max()` and `clamp()` combine several length terms with
arithmetic and folding rules (spec §4.13, "Supported"). Rather than evaluate
them at parse time — when the font size and viewport are still unknown — the
compiler lowers them into this symbolic tree.

The tree is then reduced in two steps against a
[`LengthContext`](crate::property::LengthContext):
[`LengthExpr::absolutize`] folds the font- and viewport-relative *leaves* at
the computed-value step (`calc(50% - 2em)` computes to `calc(50% - 32px)`),
and [`LengthExpr::resolve`] collapses the whole tree to a number once layout
knows the percentage basis.

The leaves are [`Length`] terms (`px` / `%` / `em` / `rem` / `vw` / `vh` /
…), so an expression can freely mix absolute, font-relative and
viewport-relative units. Multiplication and division follow the CSS rule that
one operand is a plain number: the coefficient is carried on the
[`LengthExpr::Product`] / [`LengthExpr::Quotient`] node.
