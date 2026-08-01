Computes the deterministic per-tick scroll velocity `(vx, vy)` for the
pointer at `pointer` within `viewport` (spec §6.21).

For each axis: a pointer within [`edge_margin`](Self::edge_margin) of the
low edge yields a negative velocity, within the margin of the high edge a
positive velocity, and zero otherwise. The magnitude scales linearly with
margin depth, clamped to `[0, 1]` of [`max_velocity`](Self::max_velocity).
