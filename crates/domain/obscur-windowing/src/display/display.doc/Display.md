A single monitor: its identity, geometry, video modes, and color capability
(spec §6.6).

Geometry fields are `Option` so absence can be modeled honestly. In
particular, [`Self::logical_bounds`] is `None` on platforms that do not
expose global screen coordinates — a Wayland compositor, for example, never
reveals where a monitor sits in a global logical space. `None` therefore
means "honestly unavailable", and must never be conflated with a fabricated
`(0, 0)` origin.
