Collects a node's children as [`LayoutChild`] descriptors (identity + style).

A node absent from the tree contributes no children. Each child's style is
resolved via [`style_of`] so the algorithm sees a complete descriptor.
