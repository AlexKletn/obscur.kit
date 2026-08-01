How a grid item is aligned inside its own grid area, on one axis.

css-grid-2 §11.3 (`justify-self` / `justify-items`, the inline axis) and
§11.4 (`align-self` / `align-items`, the block axis) are the same rule twice,
which is why one [`CellAlign`] serves both: the container states a default,
the item may override it, and the resolved keyword decides both how the item
is MEASURED (stretch fixes its size to the area) and where it is PLACED
within the area. Those two decisions are made in different phases of
[`GridLayout::layout`](super::GridLayout), so they have to agree; resolving
the keyword once, here, is what makes them.

The load-bearing subtlety is that `stretch` is conditional on the item's own
computed size — [`item_axis_align`] documents the condition and the half of
it this engine cannot yet test.
