Returns every entry whose bounds contain `point`, ordered topmost-first.

Ordering is by descending [`z_order`](Entry::z_order); ties are broken
deterministically by ascending [`NodeId`]. Each result's
[`local_point`](HitTest::local_point) is `point` translated into the
entry's local space (`point - bounds.origin`).
