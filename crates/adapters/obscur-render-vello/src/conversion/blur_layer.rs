//! A `filter` sub-scene extracted from the display list (spec §5.2).

use obscur_geometry::Rect;
use vello::Scene;

#[doc = include_str!("../../docs/api/conversion/blur_layer.doc/BlurLayer.md")]
pub struct BlurLayer {
    /// The bracketed subtree, as an independent scene in absolute coordinates.
    pub scene: Scene,
    /// The element's border box (the filter region), in absolute logical pixels —
    /// taken from the filter layer's clip.
    pub bounds: Rect,
    /// The blur radius in logical pixels (`0` = no blur, a color-only filter).
    pub radius: f32,
    /// The folded 4×5 color matrix (20 entries: row-major 4×4 linear part + 4-entry
    /// bias), applied to the sub-scene before any blur. [`None`] = no color op.
    pub color_matrix: Option<[f32; 20]>,
}

impl BlurLayer {
    /// Creates a filter layer descriptor carrying a blur radius and/or a color
    /// matrix.
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
