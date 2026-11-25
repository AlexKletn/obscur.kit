pub(super) struct SceneDamageBaseline {
    pub(super) list: std::sync::Arc<obscur_scene::command::DisplayList>,
    pub(super) extent: (u32, u32),
    pub(super) format: wgpu::TextureFormat,
    pub(super) text_bounds: Vec<[u32; 4]>,
}
