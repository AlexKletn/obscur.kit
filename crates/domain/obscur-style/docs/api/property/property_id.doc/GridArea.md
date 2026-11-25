The `grid-area: <custom-ident>` named-area placement on a grid item (a
`GridAreaName` marker; NOT inherited). Carried unresolved from CSS because
the area rectangle lives on the item's *container* `grid-template-areas`,
which the runtime — not the cascade — has the tree context to resolve. The
numeric `grid-area: a / b / c / d` form does not use this id; it expands to
the four `grid-*-start` / `-end` line longhands.
