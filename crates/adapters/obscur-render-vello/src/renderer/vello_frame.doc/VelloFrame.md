An offscreen [`wgpu::Texture`] and its view, sized for one frame
(spec §5.6 offscreen output).

Vello's [`render_to_texture`](vello::Renderer::render_to_texture) draws into a
texture that must have the [`wgpu::TextureFormat::Rgba8Unorm`] format and the
[`wgpu::TextureUsages::STORAGE_BINDING`] flag (Vello's fine-rasterization
stage writes it with a compute pipeline). This type allocates exactly such a
texture plus the matching [`wgpu::TextureView`] Vello binds. Additional
usages — render-attachment, texture-binding, and copy-source — let a host
blit the result to a surface or read it back for an offscreen/bitmap target.
