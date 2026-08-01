Resolves a grid item's named line placement (`grid-column: <name>` /
`grid-row: <name>`) against its container's `grid-template-columns` /
`grid-template-rows` line names, patching the item's [`LayoutStyle`] placement.

Named-line placement, like named-area placement, cannot be resolved on a single
node in isolation: a child's `grid-column: sidebar-start / main-end` names lines
declared with `[name]` groups on its **container**'s
`grid-template-columns` / `grid-template-rows`, so the lines' numbers are only
knowable with the tree in hand. The CSS layer therefore carries the child's
reference unresolved — a [`GridLineValue::Named`] edge on one of the four
`grid-*-start` / `-end` longhands — and
[`grid_placement_of`](super::layout_coordinator) maps it to
[`GridLine::Auto`]. This pass, which runs after
[`layout_styles_from`](super::LayoutCoordinator::layout_styles_from), looks each
named edge up against the parent's per-axis line-name map and overwrites just
that edge of the item's [`GridPlacement`] with the concrete
[`GridLine::Line`] the name resolves to (spec §4.13).

A child with no named edge is untouched, and a named edge whose name is absent
from the container's template (or whose parent authored no template) is left at
its [`GridLine::Auto`] default, so it simply auto-places — keeping every
non-named layout byte-for-byte identical. When a line name is declared more than
once, the **first** (lowest-numbered) occurrence wins; a trailing `<integer>`
disambiguator (`main 2`) is not yet modelled — see the crate follow-ups.
