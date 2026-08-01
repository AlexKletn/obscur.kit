Presents a submission **on-screen** by rendering into the adapter's
offscreen [`VelloFrame`] and blitting it to a windowed [`VelloSurface`].

This is the counterpart to the port [`Renderer::render`], which targets an
offscreen texture only. Vello 0.9 has no render-to-surface helper, so the
documented pattern is followed here: rasterize into an
`Rgba8Unorm + STORAGE_BINDING` intermediate (`VelloFrame::new_offscreen`)
with [`vello::Renderer::render_to_texture`], then copy that texture onto
the acquired swapchain image with a [`wgpu::util::TextureBlitter`]. The
blitter is cached on the renderer (keyed by surface format) so its render
pipeline is only compiled once, not per frame.

This is an inherent method, not part of the [`Renderer`] trait: only a host
that owns a window and a [`VelloSurface`] can drive it, whereas the trait's
offscreen path stays device- and window-agnostic.

# Errors

Returns [`RendererError::DeviceLost`] when no GPU device is attached,
[`RendererError::Backend`] when the surface has a zero extent or Vello's
rasterization fails, and surfaces any swapchain-acquisition failure
(e.g. [`RendererError::Backend`] for a lost/outdated surface).
