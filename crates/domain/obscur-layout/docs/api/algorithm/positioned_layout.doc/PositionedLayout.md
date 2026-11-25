Absolute-positioning layout (spec §4.1, §4.2).

Every child is measured at its natural size and placed at its own offset
within the container's content box, independent of its siblings — children
may overlap freely. This is the "positioned" / absolute formatting context.

# Offsets come from each child's margin

The model has no dedicated inset (`top`/`left`) properties yet, so the offset
of each child is taken from its **margin**: a child is placed at
`(padding.left + child.margin.left, padding.top + child.margin.top)`. This is
a pragmatic stand-in until dedicated positioning-offset properties exist.

# Algorithm

1. Each child is measured under `constraints` **loosened** to its container
   max, so it reports its natural size rather than being stretched.
2. Each child is placed at its margin offset inside the content box.
3. The node size is the container's own `width` / `height` when definite
   (resolved against `constraints.max`), otherwise the bounding box of the
   positioned children plus the container `padding`. The result is clamped
   into `constraints`.

# Out of scope

Containing-block resolution, right/bottom anchoring, `z-index` stacking
order, and static-position fallback are not modelled; placement is strictly
top-left offset by the child's margin.
