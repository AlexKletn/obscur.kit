#![doc = include_str!("scene_effect.doc/_module.md")]

#[doc = include_str!("scene_effect.doc/SceneEffect.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneEffect {
    #[doc = include_str!("scene_effect.doc/Gradient.md")]
    Gradient,
    #[doc = include_str!("scene_effect.doc/Clip.md")]
    Clip,
    #[doc = include_str!("scene_effect.doc/Mask.md")]
    Mask,
    #[doc = include_str!("scene_effect.doc/Image.md")]
    Image,
    #[doc = include_str!("scene_effect.doc/Text.md")]
    Text,
    #[doc = include_str!("scene_effect.doc/Blur.md")]
    Blur,
    #[doc = include_str!("scene_effect.doc/Shadow.md")]
    Shadow,
    #[doc = include_str!("scene_effect.doc/BackdropFilter.md")]
    BackdropFilter,
    #[doc = include_str!("scene_effect.doc/ColorMatrix.md")]
    ColorMatrix,
    #[doc = include_str!("scene_effect.doc/BlendMode.md")]
    BlendMode,
}

impl SceneEffect {
    #[doc = include_str!("scene_effect.doc/ALL.md")]
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

    #[doc = include_str!("scene_effect.doc/name.md")]
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
