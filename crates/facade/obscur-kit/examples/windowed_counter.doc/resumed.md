Picks up the window the **port** created, then builds the swapchain
surface and the GPU-backed renderer on it.

The window itself is not created here — `WindowedCounter::new` already
asked the port for it. What happens here is the other half of deferred
creation: forwarding to `self.handler` is what materializes the queued
OS window, and `native_window` is what hands this host the `Arc<Window>`
wgpu needs. Forward first, read second; the reverse order sees `None`.
