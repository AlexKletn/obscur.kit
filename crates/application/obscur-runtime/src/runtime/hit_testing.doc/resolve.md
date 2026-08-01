Resolves the topmost node under a hit-test point (§4.7).

`mapped` is the scroll-adjusted point in unscrolled layout space (the space
the spatial index and layout boxes are keyed by); `screen` is the point
before per-element scroll was added, in which a scroll container's clip is a
fixed on-screen rectangle. Candidates come from the spatial index (bounding
box containment); each is then admitted only if it survives
[`is_hittable`](Self::is_hittable) — overflow clipping, `border-radius`
shape — and the topmost surviving candidate wins by stacking order.

Stacking order is `(z-index, logical depth, tree pre-order)`, higher on top:
a positive `z-index` lifts a box over a lower-`z-index` sibling regardless of
document order (§4.7 "layers and z-order"), a deeper box paints over a
shallower ancestor, and among equals a later sibling paints over an earlier
one. This is a deliberate simplification of the full CSS paint-order tree —
it does not model per-stacking-context isolation or negative z-index layers —
but it is exact for the flat and singly-nested cases hit testing exercises.
