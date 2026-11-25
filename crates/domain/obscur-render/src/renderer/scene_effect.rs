//! The closed vocabulary of effects a display list can demand of a backend
//! (spec §5.2).

#[doc = include_str!("../../docs/api/renderer/scene_effect.doc/SceneEffect.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneEffect {
    /// A gradient brush (linear, radial, or conic) filling or stroking a path.
    Gradient,
    /// A geometric clip: `PushClip(Clip::Shape)`, or a shape clip carried on a
    /// layer.
    Clip,
    /// A coverage mask declaration without a coverage source.
    Mask,
    /// A coverage mask backed by an image and positioned in scene coordinates.
    ImageMask,
    /// A decoded raster image draw (`DrawImage`).
    Image,
    /// A run of pre-shaped glyphs (`DrawText`).
    Text,
    /// A gaussian blur of the element's own content: a `filter: blur()` layer, or
    /// the blur radius of a `DrawShadow`.
    Blur,
    /// A box shadow / `filter: drop-shadow()` (`DrawShadow`).
    Shadow,
    /// A blur of the already-composited backdrop (`BackdropFilter`), the
    /// frosted-glass effect.
    BackdropFilter,
    /// A folded 4×5 color matrix applied to a layer's content — the CSS color-op
    /// `filter` chain (grayscale/brightness/contrast/saturate/sepia/invert/
    /// hue-rotate/opacity).
    ColorMatrix,
    /// A non-[`Normal`](obscur_scene::paint::BlendMode::Normal) blend mode on a
    /// layer.
    BlendMode,
}

impl SceneEffect {
    /// Every effect in the vocabulary, in declaration order.
    pub const ALL: [Self; 11] = [
        Self::Gradient,
        Self::Clip,
        Self::Mask,
        Self::ImageMask,
        Self::Image,
        Self::Text,
        Self::Blur,
        Self::Shadow,
        Self::BackdropFilter,
        Self::ColorMatrix,
        Self::BlendMode,
    ];

    /// The stable name of this effect, as it appears in a
    /// [`RendererError::UnsupportedEffect`](crate::renderer::RendererError::UnsupportedEffect)
    /// and in the diagnostics built from it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Gradient => "gradient",
            Self::Clip => "clip",
            Self::Mask => "mask",
            Self::ImageMask => "image-mask",
            Self::Image => "image",
            Self::Text => "text",
            Self::Blur => "blur",
            Self::Shadow => "shadow",
            Self::BackdropFilter => "backdrop-filter",
            Self::ColorMatrix => "color-matrix",
            Self::BlendMode => "blend-mode",
        }
    }
}

impl std::fmt::Display for SceneEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_are_distinct_and_all_is_complete() {
        let names: HashSet<&str> = SceneEffect::ALL.iter().map(|e| e.name()).collect();
        assert_eq!(names.len(), SceneEffect::ALL.len());
    }

    #[test]
    fn display_is_the_name() {
        assert_eq!(SceneEffect::BackdropFilter.to_string(), "backdrop-filter");
        assert_eq!(SceneEffect::Blur.to_string(), "blur");
    }
}
