Fetches swapchain images off this thread, and owns the surface.

Present used to acquire inline, which blocked the thread winit delivers input
on for as long as the compositor took to release a buffer. See
[`SurfaceAcquirer`](crate::render_vello::renderer::SurfaceAcquirer).
