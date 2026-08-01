Reads a node's four computed `border-*-radius` corners into a [`CornerRadii`],
in top-left / top-right / bottom-right / bottom-left order, resolved against
`bounds` (the node's border box, absent for a node layout never arranged).

Relative units are already folded by the resolver (see [`border_width_of`]), so
`border-radius: .5em` rounds correctly; percentages resolve here, per axis (see
[`corner_radius_of`]). An absent radius contributes a square corner, so a node
with no authored radius yields all-zero radii — the pre-radius behaviour.
