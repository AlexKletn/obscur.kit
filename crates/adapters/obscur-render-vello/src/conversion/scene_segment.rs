//! One ordered piece of a display list split at `backdrop-filter` boundaries
//! (spec §5.2).

use vello::Scene;

use super::blur_layer::BlurLayer;

#[doc = include_str!("../../docs/api/conversion/scene_segment.doc/SceneSegment.md")]
// A `Content` segment owns a `vello::Scene` (inherently large, like `BlurLayer`),
// while `Backdrop` is a few floats; boxing the scene would only add an allocation
// to the common variant. The size difference is expected.
#[allow(clippy::large_enum_variant)]
pub enum SceneSegment {
    /// A run of drawable content (between two backdrop boundaries) as a Vello
    /// [`Scene`] plus any `filter: blur` layers extracted from it (composited on
    /// top exactly as the non-segmented path does).
    Content {
        /// The converted sub-scene for this run, in absolute coordinates.
        scene: Scene,
        /// The `filter: blur` layers extracted from this run.
        blurs: Vec<BlurLayer>,
    },
    /// A `backdrop-filter` boundary: frost (gaussian-blur) the accumulated frame
    /// within `bounds`, clipped to the rounded `radii`, by `blur` logical pixels.
    Backdrop {
        /// The panel's border box (the frost region), absolute logical pixels.
        bounds: obscur_geometry::Rect,
        /// The panel's per-corner radii (the frost is clipped to this shape).
        radii: obscur_geometry::CornerRadii,
        /// The CSS blur radius in logical pixels (`> 0`).
        blur: f32,
        /// Panel coverage intersected with the active clips.
        coverage: Scene,
    },
}

impl std::fmt::Debug for SceneSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Content { blurs, .. } => f
                .debug_struct("SceneSegment::Content")
                .field("blurs", &blurs.len())
                .field("scene", &"<vello::Scene>")
                .finish(),
            Self::Backdrop {
                bounds,
                radii,
                blur,
                ..
            } => f
                .debug_struct("SceneSegment::Backdrop")
                .field("bounds", bounds)
                .field("radii", radii)
                .field("blur", blur)
                .finish(),
        }
    }
}
