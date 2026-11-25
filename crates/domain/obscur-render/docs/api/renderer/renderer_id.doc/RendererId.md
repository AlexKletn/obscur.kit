Identifies a [`Renderer`](crate::renderer::Renderer) backend instance
(spec §5.2).

The renderer abstraction of spec §5.2 hides the base 2D backend (Vello) and
admits alternates — software/headless, platform, WASM/WebGPU, and test
renderers. When several backends coexist, this local newtype gives each a
stable identity for diagnostics and for routing frames to a chosen backend.
It is opaque data carrying no GPU or OS handle.
