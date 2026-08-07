The `pointer-events: none` value: the element is transparent to the pointer.
A pointer over its box passes straight through to whatever is painted behind
it, and the element never appears in a hit chain.

Because the property inherits, the whole subtree is transparent too — unless
a descendant declares `pointer-events: auto`, which overrides the inherited
`none` for that descendant.
