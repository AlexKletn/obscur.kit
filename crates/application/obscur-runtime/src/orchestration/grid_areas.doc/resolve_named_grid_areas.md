Resolves every grid item's `grid-area: <name>` marker against its container's
`grid-template-areas`, patching the item's [`LayoutStyle`] placement in place.

For each node carrying a [`PropertyId::GridArea`] name, the node's **logical
parent** supplies the containing grid; its computed
[`PropertyId::GridTemplateAreas`] template is looked up for the named area, and
the area's line-number rectangle is written onto the item's `grid_placement`.
A node without a marker, without a logical parent, whose parent authored no
template, or that names an unknown area is left unchanged (auto placement).

This never adds or removes entries: it only overwrites the `grid_placement`
field of items already present in `styles`, so it is a pure refinement of the
table [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from)
built. Patches are independent per node, so the (unordered) iteration is
deterministic in its result.

[`LayoutStyle`]: obscur_layout::model::LayoutStyle
