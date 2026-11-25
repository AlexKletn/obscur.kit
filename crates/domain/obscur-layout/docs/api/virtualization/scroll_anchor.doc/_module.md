The [`ScrollAnchor`] — preserves scroll position across item insertions and
resizes by pinning the viewport to a specific item (spec §4.9 *scroll
anchoring*, *stable item identity*).

When items above the viewport are inserted or re-measured, a naive scroll
offset would make content appear to jump. Anchoring instead remembers *which*
item was at the top and how far into it the viewport sat, then recomputes the
offset that keeps that item in place once extents change.
