Registers a click handler for the rows of virtual list `id` (spec §4.9).

When a row is clicked, `handler(index, clicks)` runs with the entry's **model
index** — its place in the full collection, not among the built nodes — and the
click count (`1` single, `2` double), so an app can select on a single click
and open on a double. This is the piece that makes a virtualized list an
*interactive* list: a per-row `on:click` cannot, because every row a builder
emits shares one command symbol, so only the runtime's geometry can say which
row was hit.

# Still outside §2.3's model

Now that [`on_command`](Self::on_command) delivers the event, "a handler is
given nothing" is no longer the reason this signature exists — the *model
index* is. But the payload does not reach here: the runtime calls a virtual-row
handler directly from its pointer path rather than dispatching a command
through §2.3's capture → target → bubble walk, so a row click carries no
[`UiEvent`](crate::interaction::event::UiEvent), no listener on an ancestor can
observe it, and `prevent_default` cannot suppress it. An application that needs
the pointer's position, button or device on a row still cannot have it.
