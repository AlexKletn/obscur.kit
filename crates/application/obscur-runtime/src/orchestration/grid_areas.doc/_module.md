Resolves a grid item's `grid-area: <name>` against its container's
`grid-template-areas`, patching the item's [`LayoutStyle`] placement.

Named-area placement is the one grid feature that cannot be resolved on a
single node in isolation: a child's `grid-area: header` names a rectangle that
is defined on its **container**'s `grid-template-areas`, so the area's line
numbers are only knowable with the tree in hand. The CSS layer therefore
carries the child's reference unresolved — a [`PropertyId::GridArea`] marker
([`PropertyValue::GridAreaName`]) — and this pass, which runs after
[`layout_styles_from`](super::LayoutCoordinator::layout_styles_from) has built
the per-node [`LayoutStyle`] table, looks each marker up against the parent's
parsed [`GridAreaTemplate`] and overwrites the item's
[`grid_placement`](LayoutStyle::grid_placement) with the concrete line-based
[`GridPlacement`] the area spans (spec §4.2).

A child with no marker is untouched, and a marker that names an area absent
from the container's template (or a child whose parent authored no template)
is left at its all-[`GridLine::Auto`] default, so it simply auto-places —
keeping every non-named layout byte-for-byte identical.
