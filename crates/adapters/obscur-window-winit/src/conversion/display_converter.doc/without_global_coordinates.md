Creates a converter for a platform that does not expose global monitor
coordinates — Wayland (spec §6.6).

The projected displays carry no `logical_bounds` and no
`physical_bounds`: winit's `(0, 0)` position is unavailability, and the
domain models unavailable geometry as `None`.
