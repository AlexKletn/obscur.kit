Resolves the topmost node under a hit-test point (§4.7).

`screen` is the pointer in content space — the space a fixed toolbar's box
is stated in, before any scroll is folded back. Every candidate is judged
at ITS point, [`folded_point`](Self::folded_point), which adds back only
the scroll containers that candidate actually rides; `scrolls` maps each
scroll container to its clamped `scroll_top`. Candidates come from the
spatial index — the broad phase, bounding-box containment at each of
[`query_points`](Self::query_points) — and each is then admitted only if it
survives the narrow phase: [`is_hittable`](Self::is_hittable) for
pointer-event policy, overflow clipping and `border-radius` shape, plus a
canvas's declared region. The topmost survivor wins by paint order.

A CSS `transform` on the node or any logical ancestor splits those two
phases apart. The broad phase stays in screen space (the index holds the
bounding box of the transformed corners, so it still prunes); the narrow
phase runs in the node's own space, reached by mapping the point through
the inverse of the accumulated chain. Testing the raw screen point against
the pre-transform box instead — as this did before — makes a rotated card
clickable in the empty corners of its bounding box and dead across parts of
its visible body.

# Paint order is the order

The ranking key is `(effective z, paint layer, logical depth, pre-order)`,
higher on top, and the first two are **the compositor's own**:
`effective_stacking_z` and `effective_paint_layer`, the same maps
`SceneCoordinator` sorts its paint units by. That is the point — what the
user sees on top and what the user clicks are one question, so they may not
be answered by two orderings. Reusing compose's maps buys three things a
local reading of `z-index` did not have:

* **`z-index` is per stacking context.** A node inherits its nearest
  positioned ancestor's `z`, so a plain `<div>` inside a `z-index: 10`
  panel ranks at 10 and outranks a `z-index: 5` box in a `z-index: 0`
  sibling — which is how it paints.
* **`z-index` on an unpositioned box is ignored**, per CSS 2.1 §9.9.1: it
  applies to positioned elements only.
* **A positioned box lifts above non-positioned content at the same `z`**
  (§9.9 layer 6 over layer 3), which no reading of `z-index` alone can
  express, since both are `auto`.

Two gaps are inherited deliberately, because they are compose's and the two
must not drift: stacking contexts are not compared pair-wise up the tree
(the innermost numeric `z` wins outright, so a `z: 1` inside a `z: 10`
panel ranks below a `z: 5` elsewhere), and a nested context's `z` does not
isolate its subtree from an outer one. Fixing either belongs in
`stacking.rs`, where both orderings would move together.

`logical depth` still precedes document order, and that is a known defect
rather than a rule: a deeper node in an earlier sibling beats a shallower
node in a later sibling that paints over it. Removing it is a one-line
change that turns the §4.7 oracle
`ch4_7_a_the_hit_chain_is_the_logical_tree_not_the_portal_target` red,
because that test's portalled node only wins by depth — portals project
nothing at paint time (see `docs/api/12-reality.md`), so document order puts
the overlay it is portalled into on top of it. It waits on that decision.
