Reads a node's **own** §9.9 paint-layer rank, or [`None`] when the node does not
establish one and should inherit its parent's rank.

A non-positioned node returns [`None`] (it participates in its parent's layer).
A **positioned** node returns [`PaintLayer::PositionedZeroOrAuto`] when its
effective own `z-index` is `auto` (or absent) or the integer `0` — the §9.9
layer-6 lift; a positioned node with a non-zero integer `z-index` returns
[`PaintLayer::NonPositioned`] (it is isolated by the primary `z` key and must
not lift above its own descendants — see [`PaintLayer`]).
