Reads a node's computed ordered `filter` op list, if present.

Returns [`None`] when the node has no computed style, no `filter` property, or
a non-filter value — the common (unauthored) case, which emits nothing.
`filter: none` (and a filter with only out-of-scope functions) is an **empty**
list. Each entry is a [`FilterOp`]: a drop-shadow (painted behind the node like
a `box-shadow` drop shadow) or a `blur(<radius>)` content blur.
