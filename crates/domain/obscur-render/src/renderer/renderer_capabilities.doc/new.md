Creates a conservative baseline profile.

The baseline is what the least-capable backend (a software renderer) can
promise: a `4096` maximum texture size, no MSAA, no HDR, no custom
shaders, offscreen output available, and a [`PixelFormat::Bgra8Unorm`]
preferred format. Backends widen it with the fluent setters.
