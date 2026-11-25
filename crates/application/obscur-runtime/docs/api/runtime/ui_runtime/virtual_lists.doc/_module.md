[`UiRuntime`]'s virtualized-collection surface (spec §4.9): registering a list's
row builder, the two per-row handlers a virtual row can carry, the realized
window a caller reads back, the model index under a screen point, and the
programmatic scroll of a list by its id.

A virtual row's handlers are here rather than beside the other handlers because a
virtual row is only reachable *through* the geometry: `VirtualLists` recovers a
click's model index from the window and the row extent it was realized at, and a
handler registered anywhere else would have nothing to be looked up by.

The last two are the ones that cannot be answered by `VirtualLists` alone, and
that is exactly why they are on the runtime rather than on it. `virtual_index_at`
needs the arranged boxes and the live scroll offsets to turn a screen point into
a row; `scroll_virtual_list` resolves the list to the container node it was
realized into and then moves it through the *ordinary* scroll path, so a
programmatic scroll and a wheel scroll move the same thing and clamp against the
same extent.
