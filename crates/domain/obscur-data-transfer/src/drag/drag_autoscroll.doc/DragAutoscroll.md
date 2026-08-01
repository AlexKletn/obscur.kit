Edge autoscroll configuration and per-tick velocity computation for a drag
(spec §6.21 autoscroll).

When a drag's pointer nears a viewport edge the surface should scroll to
reveal more content. This value configures the [`edge_margin`] — the
distance from an edge within which autoscroll engages — and the
[`max_velocity`] reached at the very edge, and computes a deterministic
per-tick [`velocity`] from the pointer position and viewport size. Velocity
scales linearly with how deep into the margin the pointer sits and is
negative toward the low edge, positive toward the high edge.

[`edge_margin`]: DragAutoscroll::edge_margin
[`max_velocity`]: DragAutoscroll::max_velocity
[`velocity`]: DragAutoscroll::velocity
