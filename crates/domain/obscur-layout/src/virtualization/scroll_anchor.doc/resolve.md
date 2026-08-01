Recomputes the scroll offset that keeps the anchored item in place under
the current `estimator` extents (spec §4.9).

This is the cumulative extent of items before [`index`](Self::index) plus
[`offset_within_item`](Self::offset_within_item), so when items above the
anchor grow or shrink the anchored item stays visually put.
