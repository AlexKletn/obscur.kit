Resolves a node's CSS 2D `transform` into the absolute-space affine that
brackets its subtree paint (spec §5.2), or [`None`] when there is no authored
transform (or it is the identity — the byte-identical no-op path).

Functions that depend on the border box are resolved and composed here into
a matrix `M`. It is wrapped about the element's **transform origin**: the
border-box position plus the resolved `transform-origin` (defaulting to the box
center). The returned affine is `translate(origin) · M · translate(-origin)`, so
a point in the box's untransformed absolute space is mapped about the origin —
exactly the CSS `transform` behaviour. A renderer multiplies it into its current
transform for the bracketed region.
