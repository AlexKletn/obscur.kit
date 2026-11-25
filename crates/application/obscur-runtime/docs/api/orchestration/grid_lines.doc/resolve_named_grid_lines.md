Resolves every grid item's named `grid-column` / `grid-row` line references
against the computed track lists and template areas of its visual parent.
Each edge resolves to the requested occurrence on its axis, defaulting
to the first match. Missing matches keep their previous placement.
Authored names on axes with automatic repeats are resolved after layout
expands the repeat.

This never adds or removes entries: it only overwrites the `grid_placement`
field of items already present in `styles`, so it is a pure refinement of the
table [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from)
built. Patches are independent per node, so the (unordered) iteration is
deterministic in its result.

[`LayoutStyle`]: obscur_layout::model::LayoutStyle
