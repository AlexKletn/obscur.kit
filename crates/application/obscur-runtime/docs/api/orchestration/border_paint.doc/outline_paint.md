Builds the [`PaintCommand`] stroking a node's CSS `outline` (a focus ring),
or [`None`] when no outline should paint.

An outline paints only when the node's `outline-style` is not `none`
(`solid` / `dashed`, `dotted` folding to dashed) AND its `outline-width` is a
positive pixel length. Unlike a border, the outline sits OUTSIDE the border
box: the stroke path is `bounds` inflated by `outline-offset + width/2` on
every side, so the outline's inner edge is `offset` px outside the border box
and the centered stroke is `width` thick. When the node has a border radius the
path is a rounded rect whose corner radii are `border_radius + offset + width/2`
(clamped non-negative), concentric with the rounded border box; otherwise it is
a plain rectangle. A `dashed` outline reuses the border dash pattern
([`dashed_border_pattern`]); the color falls back to the computed `color`
(`currentColor`) when `outline-color` is unset. The outline reserves no layout
space and is emitted so it paints on top of — and outside — the node.
