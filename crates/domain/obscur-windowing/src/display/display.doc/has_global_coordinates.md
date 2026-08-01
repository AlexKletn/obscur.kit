Returns whether this display exposes global logical coordinates.

This is `true` exactly when [`Self::logical_bounds`] is present. It is
`false` on platforms such as Wayland that do not reveal a global screen
coordinate space, letting callers respect that absence rather than assume
a fabricated origin.
