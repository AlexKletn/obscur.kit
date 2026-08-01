The node's CSS 2.1 §9.9 paint-layer rank (see [`effective_paint_layer`]),
sorted *between* `z` and `tree_index`: within one z bucket a
[`PaintLayer::PositionedZeroOrAuto`] box (layer 6) sorts after every
[`PaintLayer::NonPositioned`] unit (layers 1–5), so a positioned `z:auto`/`0`
box paints above its non-positioned siblings regardless of tree order. Every
bracket/paint unit of one node shares that node's `layer`, so a node's own
clip/transform/opacity/paint interleave is untouched (they tie on `z` and
`layer` and stay ordered by `order`).
