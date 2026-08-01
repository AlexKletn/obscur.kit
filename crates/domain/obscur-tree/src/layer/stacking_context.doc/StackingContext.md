Describes the stacking context a `LAYER_ROOT` node establishes (spec §5.1).

A stacking context groups a node and the subtree it composites, and fixes
that group's paint order relative to sibling contexts via a `z_index`:
higher values paint on top. This is the **descriptive** retained-tree model
only — no actual painting or compositing happens here; that lives in
`obscur-scene` / `obscur-render` later.
