Returns the two longhand [`PropertyId`]s (`start`, `end`) a **logical**
box-model shorthand expands into, or [`None`] for any other name (spec §4.13).

`margin-inline` / `margin-block` / `padding-inline` / `padding-block` /
`inset-inline` / `inset-block` each set the two edges of one axis, in
`<start> <end>` order — the flow-relative twin of how `margin` fans out to
four physical edges.

# Why the axis shorthands are here and the 4-value ones are not

CSS also defines `margin` / `padding` / `inset` *themselves* as accepting
logical values under `writing-mode`, but in a horizontal-tb, ltr/rtl engine
those shorthands are exactly the physical ones — which
[`box_shorthand_sides`] already expands. The axis shorthands are the ones that
say something physical CSS cannot: "both inline edges", whichever sides those
turn out to be.
