Anchoring to any combination of edges, not only to a single dock.

A platform whose docking API accepts exactly one edge can still carry a bar —
[`LayerAnchor::docked_edge`](crate::layer::LayerAnchor::docked_edge) tells it
which — but cannot carry a corner or a two-edge band, and should say so.
