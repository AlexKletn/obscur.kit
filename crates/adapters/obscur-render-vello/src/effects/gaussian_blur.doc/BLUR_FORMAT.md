The render-target texture format the blur pipelines are built for. Both the
Vello offscreen frame and the per-blur sub-scene textures use
[`wgpu::TextureFormat::Rgba8Unorm`], so a single pipeline set suffices.
