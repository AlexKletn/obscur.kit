#![doc = include_str!("scene_segment.doc/_module.md")]

use vello::Scene;

use super::blur_layer::BlurLayer;

#[doc = include_str!("scene_segment.doc/SceneSegment.md")]
// A `Content` segment owns a `vello::Scene` (inherently large, like `BlurLayer`),
// while `Backdrop` is a few floats; boxing the scene would only add an allocation
// to the common variant. The size difference is expected.
#[allow(clippy::large_enum_variant)]
pub enum SceneSegment {
    #[doc = include_str!("scene_segment.doc/Content.md")]
    Content {
        #[doc = include_str!("scene_segment.doc/scene.md")]
        scene: Scene,
        #[doc = include_str!("scene_segment.doc/blurs.md")]
        blurs: Vec<BlurLayer>,
    },
    #[doc = include_str!("scene_segment.doc/Backdrop.md")]
    Backdrop {
        #[doc = include_str!("scene_segment.doc/bounds.md")]
        bounds: obscur_geometry::Rect,
        #[doc = include_str!("scene_segment.doc/radii.md")]
        radii: obscur_geometry::CornerRadii,
        #[doc = include_str!("scene_segment.doc/blur.md")]
        blur: f32,
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
            } => f
                .debug_struct("SceneSegment::Backdrop")
                .field("bounds", bounds)
                .field("radii", radii)
                .field("blur", blur)
                .finish(),
        }
    }
}
