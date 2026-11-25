pub(super) struct CropTarget {
    pub(super) extent: (u32, u32),
    pub(super) color: wgpu::TextureView,
    pub(super) source: wgpu::BindGroup,
    pub(super) stencil: wgpu::TextureView,
}
