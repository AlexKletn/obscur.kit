One node's own paint commands (or a clip bracket), tagged with its stacking
key.

The commands are exactly the node's own background / border / text (never its
descendants'), captured in emission order — or, for a `border-radius` element,
a lone `PushClip` / `PopClip` that brackets the node's subtree. The key
`(z, layer, tree_index, order)` orders the unit against every other unit: `z`
is the node's effective stacking level, `layer` its §9.9 paint-layer rank
(lifting positioned-`z0`/`auto` boxes above non-positioned content within an
equal z bucket), `tree_index` its pre-order document position (the tie-breaker
within a z+layer bucket), and `order` a within-position phase so a subtree's
`PushClip` sorts before the node's paint and its `PopClip` sorts after the
whole subtree (see [`PaintUnit::PAINT_ORDER`]). When nothing is positioned
every `layer` is [`PaintLayer::NonPositioned`] and, with no radius, every unit
is [`PAINT_ORDER`](PaintUnit::PAINT_ORDER), so the key reduces to
`(z, tree_index)` — the pre-fix ordering, byte-for-byte.
