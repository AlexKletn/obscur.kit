Resolves the topmost node under a hit-test point (§4.7).

`mapped` is the scroll-adjusted point in unscrolled layout space (the space
the spatial index and layout boxes are keyed by); `screen` is the point
before per-element scroll was added, in which a scroll container's clip is a
fixed on-screen rectangle. Candidates come from the spatial index — the
broad phase, screen-space bounding-box containment — and each is then
admitted only if it survives the narrow phase: [`is_hittable`](Self::is_hittable)
for pointer-event policy, overflow clipping and `border-radius` shape, plus
a canvas's declared region. The topmost survivor wins by stacking order.

A CSS `transform` on the node or any logical ancestor splits those two
phases apart. The broad phase stays in screen space (the index holds the
bounding box of the transformed corners, so it still prunes); the narrow
phase runs in the node's own space, reached by mapping the point through
the inverse of the accumulated chain. Testing the raw screen point against
the pre-transform box instead — as this did before — makes a rotated card
clickable in the empty corners of its bounding box and dead across parts of
its visible body.

Stacking order is `(z-index, logical depth, tree pre-order)`, higher on top:
a positive `z-index` lifts a box over a lower-`z-index` sibling regardless of
document order (§4.7 "layers and z-order"), a deeper box paints over a
shallower ancestor, and among equals a later sibling paints over an earlier
one. This is a deliberate simplification of the full CSS paint-order tree —
it does not model per-stacking-context isolation or negative z-index layers —
but it is exact for the flat and singly-nested cases hit testing exercises.
