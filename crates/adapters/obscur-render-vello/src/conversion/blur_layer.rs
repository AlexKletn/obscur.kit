#![doc = include_str!("blur_layer.doc/_module.md")]

use obscur_geometry::Rect;
use vello::Scene;

#[doc = include_str!("blur_layer.doc/BlurLayer.md")]
pub struct BlurLayer {
    #[doc = include_str!("blur_layer.doc/scene.md")]
    pub scene: Scene,
    #[doc = include_str!("blur_layer.doc/bounds.md")]
    pub bounds: Rect,
    #[doc = include_str!("blur_layer.doc/radius.md")]
    pub radius: f32,
    #[doc = include_str!("blur_layer.doc/color_matrix.md")]
    pub color_matrix: Option<[f32; 20]>,
}

impl BlurLayer {
    #[doc = include_str!("blur_layer.doc/new.md")]
    #[must_use]
    pub const fn new(
        scene: Scene,
        bounds: Rect,
        radius: f32,
        color_matrix: Option<[f32; 20]>,
    ) -> Self {
        Self {
            scene,
            bounds,
            radius,
            color_matrix,
        }
    }
}

impl std::fmt::Debug for BlurLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `vello::Scene` is not `Debug`; summarise the descriptor instead.
        f.debug_struct("BlurLayer")
            .field("bounds", &self.bounds)
            .field("radius", &self.radius)
            .field("color_matrix", &self.color_matrix.is_some())
            .field("scene", &"<vello::Scene>")
            .finish()
    }
}
