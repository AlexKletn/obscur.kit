Builds a target from a live display and surface pointer.

# Safety

Both pointers must name live Wayland objects that outlive every
`wgpu::Surface` built from this target.
