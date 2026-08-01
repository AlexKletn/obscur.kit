The render-target texture format the color-matrix pipeline is built for —
matching [`GaussianBlur`](super::GaussianBlur)'s and the Vello offscreen frame's
[`wgpu::TextureFormat::Rgba8Unorm`], so the passes chain without a reformat.
