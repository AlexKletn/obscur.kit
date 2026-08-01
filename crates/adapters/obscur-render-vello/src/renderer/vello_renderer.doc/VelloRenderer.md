A GPU-accelerated [`Renderer`] backed by Vello and wgpu (spec §5.2).

# Optional device

The GPU device is host-provided and therefore **optional** (spec §2.7): a
freshly constructed `VelloRenderer` holds no [`VelloDevice`], and
[`Renderer::render`] then returns an honest
[`RendererError::DeviceLost`] rather than fabricating a device (which would
require the async `request_adapter` path the adapter refuses to own). A host
attaches a device with [`Self::with_device`] / [`Self::set_device`] once it
has created the wgpu context. Everything except the actual GPU submission —
identity, capabilities, effect support, scene conversion — works without a
device and is unit-tested that way.

# Effect support

The advertised capability profile is a GPU backend's: offscreen output, MSAA,
custom shaders (gating the trusted extensions of spec §5.3), no HDR. See
[`Renderer::supports_effect`] for the honest per-effect [`SupportLevel`].
