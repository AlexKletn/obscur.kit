A named, trusted custom render pass gated by a capability check (spec §5.3).

Spec §5.3 permits trusted extensions to inject custom render passes, but only
behind capability checks — in this backend, the
[`custom_shaders`](obscur_render::renderer::RendererCapabilities::has_custom_shaders)
capability plus a live GPU device. A real pass records wgpu commands against
the device; since the device is host-provided and optional (spec §2.7), this
type is the descriptor half: it names the pass and records whether it needs a
device, so the runtime can decide up front whether the pass is permitted.

Wiring an actual `wgpu::RenderPass` is a future refinement; the name and
permission contract here is the stable surface a trusted extension registers
against.
