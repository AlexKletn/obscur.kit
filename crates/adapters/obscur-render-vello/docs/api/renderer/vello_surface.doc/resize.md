Resizes the swapchain to `width` × `height` and reconfigures it.

The extent is clamped to a minimum of `1` in each axis, since wgpu rejects
a zero-sized surface.
