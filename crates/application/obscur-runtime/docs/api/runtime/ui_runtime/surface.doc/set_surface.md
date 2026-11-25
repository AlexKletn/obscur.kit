Configures the runtime for a window surface in one call: sets the device
[`scale`](Self::set_device_scale) and derives the layout
[`viewport`](Self::set_viewport) from the surface's **physical** extent.

This is the standard HiDPI mapping every windowed host needs (spec §5.2),
so a host does not hand-roll (and drift on) it: `scale` is the display
density (a windowing layer's `scale_factor`, optionally times a user zoom)
and the viewport is that physical extent in **logical** CSS pixels
(`physical / scale`). The UI therefore renders at its authored size for the
display density and reflows to the real window size on resize, rather than
being scaled to fit the window. The viewport divides by the *clamped*
device scale, so the two stay consistent.

Does not itself schedule a frame; pair with
[`request_frame`](Self::request_frame)`(`[`FrameReason::WindowChange`]`)`.
