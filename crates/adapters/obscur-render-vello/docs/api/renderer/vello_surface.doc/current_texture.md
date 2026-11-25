Acquires the next swapchain texture to render into.

A suboptimal texture is still returned (the caller may reconfigure at
leisure); a timed-out, occluded, outdated, lost, or validation-failed
acquisition is surfaced as an error so the caller can reconfigure and
retry.

# Errors

Returns [`VelloRenderError::SurfaceLost`] when the swapchain texture cannot
be acquired for presentation.
