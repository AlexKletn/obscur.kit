Lifecycle and geometry events waiting to be taken by the port.

Separate from [`input`](Self::input) because they leave through a different
door: these are the portable half, and a host reads them through
[`LayerBackend`](obscur_windowing::port::LayerBackend) without knowing which
platform produced them.
