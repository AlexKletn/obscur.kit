Every registered virtual list (spec §4.9): its row builder, its row and
long-press handlers, and the geometry of the window it realized last frame.

# Why these are one entity

A list's realized **window**, the **row extent** it was realized against and
the **container** it realized into are one fact — recorded by one operation
and read back by another. [`realize`](Self::realize) asks `VirtualLayout` for
the window against the container's live offset and records the extent it used
*in the same closure*, because a window without its extent is a window nobody
can point at: [`index_at`](Self::index_at) recovers a model index by dividing
a click's offset into the container by that extent, and with no extent it
simply `continue`s. Every row of that list would stop responding — to clicks
and to long presses alike, since both resolve through the same three fields —
and nothing anywhere would report it.

The second pairing is an **ordering**. A list whose scroll-reset token changed
has had its content replaced, so the offset in its container belongs to the
old content and must be zeroed *before* the window is computed from it. Both
halves are here, in one call, so there is no order left to get wrong: the
tokens are read against [`containers`](Self::containers), the offsets are
seated, and the windows are picked from what the seating left behind.

# What it deliberately does not do

It holds neither the tree, the layout boxes nor the scroll map.
[`track_containers`](Self::track_containers) takes the tree,
[`index_at`](Self::index_at) takes the boxes, and [`realize`](Self::realize)
takes the [`ScrollOffsets`] it must both reset and read — the same choice
[`ScrollOffsets`] itself makes about the live layout it clamps against. A
handler that ran is *reported* rather than acted on
([`fire_row_click`](Self::fire_row_click) returns whether one did), because
scheduling the frame that shows its effect belongs to whoever owns the
scheduler.

[`size_spacers`](Self::size_spacers) is the one member that touches none of
these fields, and it is here because it finishes what they start: the window's
extent is only *true* if the spacers bracketing it are sized to the rows they
stand in for. That pairing cannot be enforced from here — the spacer's extent
rides on the realized tree nodes, written by
[`realize_virtual_lists`](obscur_view::view::realize_virtual_lists) — so it is
stated rather than guarded.
