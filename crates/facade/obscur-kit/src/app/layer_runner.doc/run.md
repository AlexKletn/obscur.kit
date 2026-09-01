Open the desktop surface and drive it until it closes.

Waits for the first configure before drawing — the protocol requires an initial
commit with no buffer, and the platform's answer is where the real size comes
from, so the runtime is built against a viewport that is already correct rather
than one that has to be corrected.

Lifecycle and geometry come through
[`LayerBackend`](crate::windowing::port::LayerBackend); input and the render
target come from the concrete backend, exactly as the windowed runner takes
input from winit rather than through
[`WindowBackend`](crate::windowing::port::WindowBackend). One pump feeds both.
