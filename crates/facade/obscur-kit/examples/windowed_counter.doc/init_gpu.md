Initialises a wgpu swapchain surface + device/queue for **on-screen**
rendering, from an `Arc<Window>`.

The window handle is cloned into `Instance::create_surface`, yielding a
`Surface<'static>`; the adapter is requested compatible with that surface,
and the surface's default configuration is derived from the current window
size. wgpu's async `request_adapter` / `request_device` are driven to
completion with `pollster::block_on`.

# Errors

Returns an error if the surface cannot be created, no compatible GPU adapter
is available (as in a headless environment), the device cannot be created, or
the surface exposes no supported default configuration.
