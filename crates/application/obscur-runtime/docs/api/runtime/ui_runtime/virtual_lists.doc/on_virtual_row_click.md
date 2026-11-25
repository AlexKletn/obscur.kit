Registers a click handler for the rows of virtual list `id` (spec §4.9).

When a trusted pointer click lands inside that list's realized container, the
runtime resolves the click to the row's **model index** — not the index of a
built node, but the item's place in the full collection — and calls
`handler(index, clicks)`, where `clicks` is `1` for a single click and `2` for
a double.

This exists because a virtual row cannot carry a per-row `on-click`: every row
realized from one builder lowers to the *same* command symbol, so the symbol
alone can never say which row was hit. The index can only be recovered from the
container's box, its scroll offset and the row extent — which the runtime owns
and the app, having deliberately not built the rows, does not.
