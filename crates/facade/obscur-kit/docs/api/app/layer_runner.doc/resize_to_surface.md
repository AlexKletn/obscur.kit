Validates the physical extent before applying the latest surface geometry.
Wayland receives the buffer scale before the runtime and swapchain adopt the
new dimensions, so the compositor interprets the next buffer at the right size.
