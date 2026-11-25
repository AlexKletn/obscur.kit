Normal-flow block layout for a **vertical** writing mode (css-writing-modes-4
§6.2): the block axis is horizontal, so in-flow children stack left/right
instead of top/bottom.

The block axis is physical **width**; the inline axis is physical **height**.
`vertical-lr` progresses left-to-right from the left content edge;
`vertical-rl` progresses right-to-left from the right content edge. Children's
logical block-start / block-end margins are read from the physical edges the
mode maps them to (already folded from `margin-block-*` upstream by the style
crate): for `vertical-rl` block-start is the right edge (`margin.right`), for
`vertical-lr` it is the left edge (`margin.left`). Block-axis margin collapsing
(CSS 2.1 §8.3.1) and the non-collapsing `gap` work exactly as in the horizontal
path, only along the rotated axis.

The inline (vertical) axis carries each child's used height and its top/bottom
margins, resolved as the §10.3.3 inline solve on the vertical axis.

Percentage *sizes* resolve against the containing block on both axes, exactly as
in the horizontal path and for the same reason — see [`FlowLayout`]'s note on why
that is not the remaining extent. Here the axis that used to compound across
siblings is the physical **width**: two `width: 50%` children of a 480-wide
`vertical-lr` container are 240 each, where the second used to read half of the
240 that remained.

# Modelled to the point the block-progression is correct

Intrinsic inline sizing along the vertical axis (`height: max-content` on a
vertical container) is not probed — a vertical `auto` inline size stretches to
the offer, and an intrinsic min/max-height bound degrades to `auto` — mirroring
the deliberate one-axis limit of [`intrinsic_inline_size`]. The block axis
(width) shrink-wraps its children when `width: auto`.
