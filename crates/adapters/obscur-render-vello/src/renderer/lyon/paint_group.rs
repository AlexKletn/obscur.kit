use super::mesh_mask::ImageMask;

pub(super) struct PaintGroup {
    pub(super) mask: Option<ImageMask>,
    pub(super) blend: Option<u32>,
    pub(super) replacement_bounds: Option<[u32; 4]>,
    pub(super) opacity: f32,
    pub(super) vertex_start: usize,
    pub(super) color_filter: Option<u32>,
    pub(super) blur_radius: f32,
    pub(super) preceding_blurs: usize,
}
