Input the compositor delivered to a layer surface.

Separate from
[`LayerSurfaceEvent`](obscur_windowing::layer::LayerSurfaceEvent), and
deliberately so. Lifecycle and geometry are portable — every platform has a
notion of *configured*, *resized*, *closed* — and travel through the port.
Input is not the port's business at all: it reaches the runtime the same way a
window's input does, so a panel behaves like every other surface in the toolkit
rather than growing a second input path of its own.

Both queues drain from one pump. `dispatch` turns the loop; this and
[`LayerBackend::poll_events`](obscur_windowing::port::LayerBackend::poll_events)
take what it produced.
