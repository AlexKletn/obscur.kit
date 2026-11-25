The [`VirtualLayout`] — computes which items to materialize for a scroll
position (spec §4.9 *virtualization and collections*).

This is the reusable 1-D virtualization mechanism the higher-level collections
(tables, trees, grids) compose: given a scroll offset and viewport extent it
resolves the visible item band via the [`SizeEstimator`], widens it by the
[`OverscanPolicy`], and clamps the result to a [`MaterializedRange`]. It also
captures [`ScrollAnchor`]s for stable scroll positions across resizes.
