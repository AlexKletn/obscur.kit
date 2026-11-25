Registers a long-press handler for the rows of virtual list `id` (spec
§4.9, §6.16).

The touch counterpart to [`on_virtual_row_click`](Self::on_virtual_row_click),
and it exists for the same reason: a realized row cannot carry a per-row
binding, so the model index has to come from the geometry the runtime
owns. Without it, the one gesture that reaches a context action on a
tablet could reach every element in an application *except* the rows of a
virtual list — which, in a file manager, is the only thing anyone wants to
act on.

Called once, when the hold is recognized, while the finger is still down;
the lift that follows is consumed and is not a click.
