Resolves the topmost node under a hit-test point (§4.7).

`screen` is the pointer in content space — the space a fixed toolbar's box
is stated in, before any scroll is folded back. Every candidate is judged
at ITS point, [`folded_point`](Self::folded_point), which adds back only
the scroll containers that candidate actually rides; `scrolls` maps each
scroll container to its clamped `scroll_top`. Candidates come from the
spatial index — the broad phase, bounding-box containment at each of
[`query_points`](Self::query_points) — and each is then admitted only if it
survives the narrow phase: [`is_hittable`](Self::is_hittable) for
pointer-event policy, the visual ancestors’ overflow and rounded clips,
the target’s rounded border box, and the canvas’s declared region. The topmost survivor wins by paint order.

A CSS `transform` on the node or any logical ancestor splits those two
phases apart. The broad phase stays in screen space (the index holds the
bounding box of the transformed corners, so it still prunes); the narrow
phase runs in the node's own space, reached by mapping the point through
the inverse of the accumulated chain. Testing the raw screen point against
the pre-transform box instead — as this did before — makes a rotated card
clickable in the empty corners of its bounding box and dead across parts of
its visible body.

# Paint order is the order

Hit testing compares the resolved stacking band and paint pre-order.
Each stacking context is ranked as one group in its parent context. Numeric
z-index, non-none transforms and filters, non-normal mixing, and opacity
below one establish contexts. Fixed and sticky positioning also establish
a context with auto z-index.
The order follows the same visual traversal used for composition, including
CSS order groups for Grid and Flexbox children and items promoted through
boxless wrappers.
