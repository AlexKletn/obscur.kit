Absolutizes every leaf against `ctx`, keeping the expression's shape.

This is [`Length::absolutize`] lifted over the tree: the font- and
viewport-relative leaves become `px` while `%`, `auto` and an unmeasured
`ch` stay symbolic, so `calc(50% - 2em)` computes to `calc(50% - 32px)`
exactly as CSS specifies. The tree is *not* folded to a single number —
that needs the percentage basis, which only layout has.
