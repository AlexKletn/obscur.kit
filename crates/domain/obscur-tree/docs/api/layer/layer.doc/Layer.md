A compositing layer that groups a subtree for composition (spec §5.1).

A layer pairs an opaque [`LayerId`] with the establishing `root` node and the
[`StackingContext`] that fixes its paint order among siblings. This is the
**retained-tree-side descriptor**: it names and orders a layer, but the
actual compositing (surface allocation, blending, GPU work) lives in
`obscur-scene` / `obscur-render` later.
