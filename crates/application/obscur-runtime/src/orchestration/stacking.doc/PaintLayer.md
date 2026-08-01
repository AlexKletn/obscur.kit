A node's CSS 2.1 §9.9 / Appendix E **paint-layer rank** — the order key used
*within a single z bucket* to lift positioned boxes above non-positioned
content.

§9.9.1 paints a stacking context bottom→top in seven layers; layers 2 (negative
z) and 7 (positive z) are already separated here by the primary effective-`z`
key, so this rank only distinguishes the `z == 0` layers:

* [`NonPositioned`](Self::NonPositioned) — the context root's background &
  borders (layer 1) and every non-positioned in-flow **block** (3), **float**
  (4) and **inline** (5) descendant. These four §9.9 layers are **collapsed
  into one rank** because `float` is not a computed property in this engine and
  because splitting block (3) from inline (5) would reorder non-positioned
  siblings out of document order — violating the byte-identical regression
  guard. Within this rank `tree_index` (document order) decides.
* [`PositionedZeroOrAuto`](Self::PositionedZeroOrAuto) — a **positioned**
  descendant (and, by inheritance, its subtree) whose effective own `z-index`
  is `auto` or `0` (§9.9 layer 6). It sorts **after** every `NonPositioned`
  unit in the same z bucket, so the whole positioned box paints on top of its
  non-positioned siblings even when it comes earlier in the tree — the fix for
  catalog bug B2.

A positioned box with a non-zero integer `z-index` is *not* lifted here: the
primary `z` key already floats it into its own bucket (layer 2 or 7), and
keeping it at `NonPositioned` leaves it below its own descendants (which share
its z) in document order — matching "the SC root's own background paints first"
(§9.9 layer 1 within its own context).
