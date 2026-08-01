What a **child's** percentage size resolves against, as the algorithm that
placed that child states it — one optional component per axis.

# Why the runtime cannot work this out for itself

The runtime's arrange walk hands every child of a node the same percentage
basis: the node's own content box. That is the containing block for a block
child (CSS 2.1 §10.1) and for a flex item (css-flexbox-1 §4), so for those
formatting contexts it is exactly right and this type is never stated.

It is **not** right for a grid item. `specs/w3c/css-grid-2.txt:712`, vendored
here: "A grid item's grid area forms the containing block into which it is
laid out", restated at `:1042` — "A grid item is sized within the containing
block defined by its grid area." The grid area is a per-item rectangle carved
out of the resolved tracks; only the grid algorithm knows it, and it differs
from the container's content box for every grid that has more than one track.
Handing the container's content box down instead is how a `width: 100%` grid
item came out at the track width (correct) while its children resolved their
percentages against the whole grid container's content width — the item's used
size and the size its children saw disagreed
(`docs/external-consumer-audit.md`, 2-4 (a)).

# Why per axis, and not a [`Size`]

[`BoxConstraints::percent_basis`] is a single [`Size`], so a caller either
states both axes or neither — which is why [`FlowLayout`] attaches its basis
only when its *block* axis is definite and lets a perfectly well-known inline
basis fall back with it. A grid is the mirror image: an item's grid area has a
resolved inline extent by the time any item is laid out, while its block
extent may be the item's own content (an `auto` row is sized *by* the items in
it), and a containing block derived from the box it contains is no containing
block at all. Stating one axis and leaving the other alone is the only honest
answer, and it is what keeps a fix in one axis from silently moving the other.

[`BoxConstraints::percent_basis`]: obscur_geometry::BoxConstraints::percent_basis
[`FlowLayout`]: super::FlowLayout
