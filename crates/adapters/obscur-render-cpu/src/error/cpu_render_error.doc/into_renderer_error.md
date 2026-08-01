Lifts this adapter error into the domain [`RendererError`] the
[`Renderer`](obscur_render::Renderer) port returns (spec §5.2).

A buffer that cannot be allocated and a zero-area target are both
[`RendererError::InvalidTarget`]: the target the caller asked for cannot
hold a frame. A [`CpuRenderError::Backend`] failure maps to
[`RendererError::Backend`], preserving its reason.
