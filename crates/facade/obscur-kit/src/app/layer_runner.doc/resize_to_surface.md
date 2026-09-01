Configures the swapchain at the window's current physical extent.

Called on every configure, and once before the loop — the first time is not
redundant. `init_gpu` configures the surface before `sync_scale` has said how
the buffer relates to logical pixels, and a swapchain built against that answer
presents at double extent until something reconfigures it (audit 1-E).
