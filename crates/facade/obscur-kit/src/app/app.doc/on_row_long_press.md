Registers a **long-press** handler for the rows of virtual list `id`: a
finger held still on a row calls `handler(model_index)` while it is still
down, and the lift that follows is not a click.

The touch counterpart to [`on_row_click`](Self::on_row_click), and the only
way a virtualized row reaches a context action: a touchscreen has one
button and no modifiers, so a hold is what a secondary click would have
been — and a realized row cannot carry a per-row binding to receive it.

Outside §2.3's propagation for the same reason as
[`on_row_click`](Self::on_row_click): the hold is recognised on the runtime's
pointer path and calls this directly, so no event is delivered and no listener
sees it.
