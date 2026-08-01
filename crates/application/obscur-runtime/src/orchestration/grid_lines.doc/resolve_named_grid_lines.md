Resolves every grid item's named `grid-column` / `grid-row` line references
against its container's `grid-template-columns` / `grid-template-rows` line
names, patching the item's [`LayoutStyle`] placement in place.

For each node carrying a [`GridLineValue::Named`] edge, the node's **logical
parent** supplies the containing grid; the parent's computed track lists provide
the per-axis `name -> line number` map, and each named edge is rewritten to the
concrete [`GridLine::Line`] it resolves to. A node without a named edge, without
a logical parent, whose parent authored no line names, or that names an unknown
line is left unchanged (auto placement).

This never adds or removes entries: it only overwrites the `grid_placement`
field of items already present in `styles`, so it is a pure refinement of the
table [`layout_styles_from`](super::LayoutCoordinator::layout_styles_from)
built. Patches are independent per node, so the (unordered) iteration is
deterministic in its result.

[`LayoutStyle`]: obscur_layout::model::LayoutStyle
