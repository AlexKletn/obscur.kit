Resolves the computed `grid-auto-flow` keyword to a [`GridAutoFlow`],
defaulting to [`GridAutoFlow::Row`].

Compares the interned keyword symbol against the reserved [`layout_keyword`]
`GRID_AUTO_FLOW_*` handles (each authored combination folds to one symbol);
an absent or unrecognized value maps to [`GridAutoFlow::Row`].
