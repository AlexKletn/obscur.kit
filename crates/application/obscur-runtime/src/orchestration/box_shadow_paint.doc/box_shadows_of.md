Reads a node's computed `box-shadow` list, if present.

Returns [`None`] when the node has no computed style, no `box-shadow`
property, or a non-shadow value — the common (unauthored) case, which emits
no shadows. `box-shadow: none` is an **empty** list (also nothing to emit).
