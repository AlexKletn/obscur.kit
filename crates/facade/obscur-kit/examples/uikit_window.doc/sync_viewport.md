Re-syncs the runtime's layout viewport and device scale to the current
swapchain extent, display density, and user zoom. Called after surface
creation, on resize, on a scale-factor change, and after a zoom change.

The viewport is the swapchain's **physical** extent divided by the effective
scale, so layout runs in **logical** CSS pixels; the runtime's device scale
is set to that same factor, so the composed scene is bracketed by a root
`scale(s, s)` and rasterizes crisply at physical resolution.
