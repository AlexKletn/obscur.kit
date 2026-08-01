Computes each node's **effective §9.9 paint-layer rank** for the whole tree.

Walks pre-order from the root (cycle-safe via a visited set); the root's rank
is [`PaintLayer::NonPositioned`]. Each node's rank is its **own** rank when it
is positioned (see [`own_paint_layer`]), otherwise it **inherits** its parent's
rank. Inheritance makes a positioned `z:auto`/`0` box's *whole subtree* share
the [`PositionedZeroOrAuto`](PaintLayer::PositionedZeroOrAuto) lift, so the
entire positioned box (background, borders and non-positioned descendants) rises
above non-positioned siblings **together**, while `tree_index` keeps the
ancestor painting below its own descendants. (Nested-context re-isolation — a
deeper positioned integer-z context resetting the rank — is applied when that
child is itself positioned, but a descendant's high z otherwise escaping its
parent context stays the documented simplification.)

When no node is positioned every rank is
[`NonPositioned`](PaintLayer::NonPositioned), so the layer key is uniform and
the sort reduces to `tree_index` order — the byte-identical regression guard.
