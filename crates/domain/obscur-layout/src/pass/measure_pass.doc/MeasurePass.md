The **measure** phase input for a node (spec §4.2): the [`BoxConstraints`]
under which the node is asked for its desired size.

Measure returns a desired size — a plain [`Size`](obscur_geometry::Size),
which is what the runtime's measure pass and its cache actually carry —
without positioning anything; positioning is the job of the separate arrange
phase ([`ArrangePass`](super::ArrangePass)).
