A failure raised inside the software [`CpuRenderer`](crate::renderer::CpuRenderer)
while rasterizing a frame (spec §5.2).

This is the adapter's own, `tiny-skia`-shaped error. It never leaves the
adapter as-is: [`CpuRenderError::into_renderer_error`] lifts it into the
backend-agnostic [`RendererError`] the `Renderer` port speaks, so callers see
only the domain diagnostic. Every variant is an explicit, actionable cause,
honoring the rule of spec §5.2 that a backend reports failures rather than
panicking.
