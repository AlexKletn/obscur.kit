A [`wgpu::Surface`] with its [`wgpu::SurfaceConfiguration`], for on-screen
presentation (spec §5.2).

# Where the surface comes from

Building a surface requires a window's raw-window-handle and the wgpu
`Instance`, both owned by the *host* (spec §2.7): the adapter never touches a
window. The host creates a `Surface<'static>` and its configuration and passes
them to [`VelloSurface::new`]; this type then owns the swapchain lifecycle
(configure, resize, acquire) that a renderer drives each frame.

The surface is `'static` because it must outlive individual frames; the host
guarantees the window lives at least as long.
