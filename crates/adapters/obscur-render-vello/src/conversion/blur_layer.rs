//! A blur sub-scene extracted from the display list (spec §5.2).

use obscur_geometry::Rect;
use vello::Scene;

/// One `filter: blur(radius)` layer, extracted from the display list as a separate
/// sub-scene the renderer blurs and composites (spec §5.2).
///
/// The scene coordinator brackets a blurred element's subtree with a
/// `PushLayer(Blur)` … `PopLayer`; [`SceneConverter::convert_with_blurs`] detects
/// that bracket, converts the bracketed commands into their **own** [`Scene`]
/// (omitting them from the main scene so they are not also drawn un-blurred), and
/// records this descriptor. The renderer then renders [`scene`](Self::scene) into
/// an offscreen texture, runs a separable gaussian pass over it, and composites the
/// blurred result back onto the target at [`bounds`](Self::bounds).
///
/// The sub-scene draws in the **same absolute coordinates** as the main scene (any
/// enclosing transform is already baked in), so the renderer offsets it to a
/// margin-padded offscreen texture and composites it back at its absolute position.
pub struct BlurLayer {
    /// The bracketed subtree, as an independent scene in absolute coordinates.
    pub scene: Scene,
    /// The element's border box (the blur region), in absolute logical pixels —
    /// taken from the blur layer's clip.
    pub bounds: Rect,
    /// The blur radius in logical pixels (`> 0`).
    pub radius: f32,
}

impl BlurLayer {
    /// Creates a blur layer descriptor.
    #[must_use]
    pub const fn new(scene: Scene, bounds: Rect, radius: f32) -> Self {
        Self {
            scene,
            bounds,
            radius,
        }
    }
}

impl std::fmt::Debug for BlurLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `vello::Scene` is not `Debug`; summarise the descriptor instead.
        f.debug_struct("BlurLayer")
            .field("bounds", &self.bounds)
            .field("radius", &self.radius)
            .field("scene", &"<vello::Scene>")
            .finish()
    }
}
