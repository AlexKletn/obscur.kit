Returns every entry whose bounds contain `point`, ordered topmost-first.

Ordering is by descending [`z_order`](Entry::z_order); ties are broken
deterministically by ascending [`NodeId`]. Each result's
[`local_point`](HitTest::local_point) is `point` translated into the
entry's local space (`point - bounds.origin`).

This is a **broad phase**: [`bounds`](Entry::bounds) is an axis-aligned
rect, so an entry whose real shape is not (a rotated element, whose
bounds are the box around its corners) is over-reported here and must be
narrowed by the caller — which also means `local_point` is only the
element's own coordinate space when the mapping between them is a pure
translation.
