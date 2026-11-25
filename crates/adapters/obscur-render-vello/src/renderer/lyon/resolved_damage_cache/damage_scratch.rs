use super::{SAMPLES, STENCIL, texture};

pub(super) struct DamageScratch {
    pub(super) extent: (u32, u32),
    pub(super) resolved: wgpu::Texture,
    pub(super) resolve_view: wgpu::TextureView,
    pub(super) color: wgpu::TextureView,
    pub(super) stencil: wgpu::TextureView,
}

impl DamageScratch {
    pub(super) fn new(
        device: &wgpu::Device,
        extent: (u32, u32),
        format: wgpu::TextureFormat,
    ) -> Self {
        let transient = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TRANSIENT;
        let resolved = texture(
            device,
            extent,
            format,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let resolve_view = resolved.create_view(&Default::default());
        Self {
            extent,
            resolved,
            resolve_view,
            color: texture(device, extent, format, SAMPLES, transient)
                .create_view(&Default::default()),
            stencil: texture(device, extent, STENCIL, SAMPLES, transient)
                .create_view(&Default::default()),
        }
    }
}
