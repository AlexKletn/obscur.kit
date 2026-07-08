//! A `filter` sub-scene extracted from the display list (spec §5.2).

use obscur_geometry::Rect;
use vello::Scene;

/// One `filter` layer — a `blur(radius)` and/or a color-matrix transform —
/// extracted from the display list as a separate sub-scene the renderer
/// post-processes and composites (spec §5.2).
///
/// The scene coordinator brackets a filtered element's subtree with a
/// `PushLayer(filter)` … `PopLayer`; [`SceneConverter::convert_with_blurs`] detects
/// that bracket, converts the bracketed commands into their **own** [`Scene`]
/// (omitting them from the main scene so they are not also drawn unfiltered), and
/// records this descriptor. The renderer then renders [`scene`](Self::scene) into
/// an offscreen texture, applies the owned color-matrix pass (if
/// [`color_matrix`](Self::color_matrix) is set) and/or a separable gaussian pass
/// (if [`radius`](Self::radius) `> 0`), and composites the result back onto the
/// target at [`bounds`](Self::bounds).
///
/// The sub-scene draws in the **same absolute coordinates** as the main scene (any
/// enclosing transform is already baked in), so the renderer offsets it to a
/// margin-padded offscreen texture and composites it back at its absolute position.
///
/// Despite the historical name, a layer may carry a color matrix with **no** blur
/// (`radius == 0`) — the color-only filter path.
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
