The node's positioning scheme (`position`).

[`Position::Static`] (the default) lays the node out in normal flow and
ignores [`inset`](Self::inset). [`Position::Relative`] keeps the node in
flow but offsets its final position (and its whole subtree) by
[`inset`](Self::inset). [`Position::Absolute`] / [`Position::Fixed`] take
the node **out of flow** and position it by [`inset`](Self::inset) within
its containing block (the nearest positioned ancestor's padding box, or the
viewport for `fixed`).
