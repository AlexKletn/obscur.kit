Returns the nodes whose recorded interaction state differs between
`before` and `after`.

Both maps are canonical — a node with no entry carries
[`PseudoState::empty`](obscur_style::selector::PseudoState::empty) — so the
diff walks only the two maps' entries (the hover/focus chain, a handful of
nodes), never the tree. A node whose entry was dropped is reported just as
one whose bits changed: un-hovering must restyle too.

The result is a set in all but type: each changed node appears exactly once,
and the order (which follows hash iteration) is not observable — [`expand`](Self::expand)
unions the reaches.
