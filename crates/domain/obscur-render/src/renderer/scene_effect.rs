//! The closed vocabulary of effects a display list can demand of a backend
//! (spec §5.2).

/// One effect a [`DisplayList`](obscur_scene::command::DisplayList) can demand of
/// a [`Renderer`](crate::renderer::Renderer) (spec §5.2).
///
/// Spec §5.2 requires that "unsupported effects and fallbacks are explicit
/// diagnostics". An explicit diagnostic needs a **closed** vocabulary: a free
/// string lets a backend answer `Emulated` for a name it has never heard of, which
/// is how the CPU backend came to claim it emulated `blur` while dropping the blur
/// radius on the floor. Every variant here is something the display list actually
/// carries, so a backend's [`supports_effect`](crate::renderer::Renderer::supports_effect)
/// table is an exhaustive match the compiler checks, and
/// [`scene_effects`](crate::renderer::scene_effects) can name exactly what a given
/// scene needs.
///
/// The vocabulary covers *effects* — the marks and filters a backend either
/// reproduces or does not. It deliberately does **not** cover every compositing
/// parameter a [`SceneLayer`](obscur_scene::layer::SceneLayer) carries: group
/// opacity, the Porter-Duff operator, and a layer's own transform have no variant
/// here and are therefore not diagnosable through this port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneEffect {
    /// A gradient brush (linear, radial, or conic) filling or stroking a path.
    Gradient,
    /// A geometric clip: `PushClip(Clip::Shape)`, or a shape clip carried on a
    /// layer.
    Clip,
    /// A coverage-mask clip: `PushClip(Clip::Mask)`, or a mask clip carried on a
    /// layer.
    Mask,
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
    pub const ALL: [Self; 10] = [
        Self::Gradient,
        Self::Clip,
        Self::Mask,
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
