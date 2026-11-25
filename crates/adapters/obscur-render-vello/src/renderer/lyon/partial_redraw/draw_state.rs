use obscur_geometry::AffineTransform;

#[derive(Clone, Copy)]
pub(super) struct DrawState {
    pub(super) transform: AffineTransform,
    pub(super) clip: [f32; 4],
}
