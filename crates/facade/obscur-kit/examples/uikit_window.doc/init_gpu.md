Initialises a wgpu swapchain surface + device/queue for on-screen rendering
from an `Arc<Window>` (mirrors `showcase_window.rs`).

# Errors

Returns an error if the surface cannot be created, no compatible GPU adapter is
available (as in a headless environment), the device cannot be created, or the
surface exposes no supported default configuration.
