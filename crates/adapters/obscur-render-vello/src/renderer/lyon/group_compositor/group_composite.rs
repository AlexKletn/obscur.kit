pub(super) struct GroupComposite<'a> {
    pub(super) source: &'a wgpu::BindGroup,
    pub(super) bounds: [u32; 4],
    pub(super) parameters: u32,
    pub(super) vertex_start: u32,
    pub(super) stencil_depth: Option<u32>,
}
