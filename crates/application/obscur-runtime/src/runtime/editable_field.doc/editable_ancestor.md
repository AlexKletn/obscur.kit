Resolves the nearest editable ancestor of `target` (target first, toward
the root): the first node whose own bindings carry
[`EVENT_INPUT`](obscur_view::view::EVENT_INPUT). Returns `None` when no
node in the chain is editable.
