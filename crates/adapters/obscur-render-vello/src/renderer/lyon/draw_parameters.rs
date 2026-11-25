use obscur_geometry::AffineTransform;

#[derive(Clone, Copy)]
pub(super) struct DrawParameters {
    pub(super) transform: AffineTransform,
    pub(super) opacity: f32,
}
