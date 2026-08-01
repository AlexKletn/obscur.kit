Initialises a wgpu swapchain surface + device/queue for on-screen rendering
from an `Arc<Window>` (the standard pollster `Instance → Surface → Adapter →
Device → Queue` chain, exactly as the windowed examples do).
