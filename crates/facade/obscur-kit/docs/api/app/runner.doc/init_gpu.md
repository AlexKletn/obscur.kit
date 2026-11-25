Initialises a wgpu swapchain surface + device/queue for on-screen rendering
from an `Arc<Window>` (the standard pollster `Instance → Surface → Adapter →
Device → Queue` chain, exactly as the windowed examples do).

The first three of those come from `gpu_instance::open`, which is shared with
the layer runner: it is where the backend set is chosen, and where the reason
that is not simply `Backends::all()` is written down with its numbers.
