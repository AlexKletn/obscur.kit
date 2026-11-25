Whether a node's computed `overflow` makes it a **user-scrollable** box — the
reserved `scroll` or `auto` handles (css-overflow-3 §3) on the **block** axis.
`visible` / `hidden` / `clip` / absent are not: `hidden` and `clip` still clip,
and `hidden` can still be scrolled programmatically, but neither answers a
wheel.

Only the block axis is consulted because only the block axis can move: a box
carries one scroll offset. `overflow-x: auto` is honoured as far as clipping and
container-hood, and then stops — a limitation of the scroll model, not of the
property.
