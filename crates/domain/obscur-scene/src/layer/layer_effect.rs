#![doc = include_str!("layer_effect.doc/_module.md")]

use crate::paint::BlendMode;

#[doc = include_str!("layer_effect.doc/LayerEffect.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerEffect {
    #[doc = include_str!("layer_effect.doc/Opacity.md")]
    Opacity(f32),
    #[doc = include_str!("layer_effect.doc/Blur.md")]
    Blur {
        #[doc = include_str!("layer_effect.doc/radius.md")]
        radius: f32,
    },
    #[doc = include_str!("layer_effect.doc/Blend.md")]
    Blend(BlendMode),
    #[doc = include_str!("layer_effect.doc/ColorMatrix.md")]
    ColorMatrix([f32; 20]),
}

impl LayerEffect {
    #[doc = include_str!("layer_effect.doc/opacity.md")]
    #[must_use]
    pub const fn opacity(factor: f32) -> Self {
        Self::Opacity(factor)
    }

    #[doc = include_str!("layer_effect.doc/blur.md")]
    #[must_use]
    pub const fn blur(radius: f32) -> Self {
        Self::Blur { radius }
    }

    #[doc = include_str!("layer_effect.doc/blend.md")]
    #[must_use]
    pub const fn blend(mode: BlendMode) -> Self {
        Self::Blend(mode)
    }

    #[doc = include_str!("layer_effect.doc/color_matrix.md")]
    #[must_use]
    pub const fn color_matrix(matrix: [f32; 20]) -> Self {
        Self::ColorMatrix(matrix)
    }

    #[doc = include_str!("layer_effect.doc/is_opacity.md")]
    #[must_use]
    pub const fn is_opacity(self) -> bool {
        matches!(self, Self::Opacity(_))
    }

    #[doc = include_str!("layer_effect.doc/is_blur.md")]
    #[must_use]
    pub const fn is_blur(self) -> bool {
        matches!(self, Self::Blur { .. })
    }

    #[doc = include_str!("layer_effect.doc/is_blend.md")]
    #[must_use]
    pub const fn is_blend(self) -> bool {
        matches!(self, Self::Blend(_))
    }

    #[doc = include_str!("layer_effect.doc/is_color_matrix.md")]
    #[must_use]
    pub const fn is_color_matrix(self) -> bool {
        matches!(self, Self::ColorMatrix(_))
    }

    #[doc = include_str!("layer_effect.doc/as_color_matrix.md")]
    #[must_use]
    pub const fn as_color_matrix(self) -> Option<[f32; 20]> {
        match self {
            Self::ColorMatrix(matrix) => Some(matrix),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opacity_constructor_and_predicate() {
        let effect = LayerEffect::opacity(0.5);
        assert!(effect.is_opacity());
        assert!(!effect.is_blur());
        assert_eq!(effect, LayerEffect::Opacity(0.5));
    }

    #[test]
    fn blur_constructor_and_predicate() {
        let effect = LayerEffect::blur(3.0);
        assert!(effect.is_blur());
        assert_eq!(effect, LayerEffect::Blur { radius: 3.0 });
    }

    #[test]
    fn blend_constructor_and_predicate() {
        let effect = LayerEffect::blend(BlendMode::Multiply);
        assert!(effect.is_blend());
        assert_eq!(effect, LayerEffect::Blend(BlendMode::Multiply));
    }

    #[test]
    fn color_matrix_constructor_and_accessors() {
        let mut matrix = [0.0_f32; 20];
        // Identity 4×4 (row major) with a zero bias.
        matrix[0] = 1.0;
        matrix[5] = 1.0;
        matrix[10] = 1.0;
        matrix[15] = 1.0;
        let effect = LayerEffect::color_matrix(matrix);
        assert!(effect.is_color_matrix());
        assert!(!effect.is_blur());
        assert_eq!(effect.as_color_matrix(), Some(matrix));
        assert_eq!(LayerEffect::blur(1.0).as_color_matrix(), None);
    }
}
