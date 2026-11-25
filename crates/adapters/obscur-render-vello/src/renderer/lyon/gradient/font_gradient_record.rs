use obscur_scene::paint::{ExtendMode, GradientStop};

pub(super) struct FontGradientRecord {
    pub(super) kind: f32,
    pub(super) extend: ExtendMode,
    pub(super) geometry: [[f32; 4]; 4],
    pub(super) stops: Vec<GradientStop>,
    pub(super) index: u32,
}
