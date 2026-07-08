//! An effect applied to a layer's composited content.

use crate::paint::BlendMode;

/// An effect applied to a layer's composited content (spec §5.1).
///
/// A `LayerEffect` is a backend-independent descriptor of a post-composition
/// operation: uniform opacity, a Gaussian-style blur, a blend mode against
/// the backdrop, or a color-matrix transform. It references no GPU or renderer
/// types. Shadows are a later addition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayerEffect {
    /// Scales the layer's coverage by an opacity factor (nominally `0.0..=1.0`).
    Opacity(f32),
    /// Blurs the layer's content with the given radius in logical pixels.
    Blur {
        /// The blur radius in logical pixels.
        radius: f32,
    },
    /// Blends the layer against its backdrop with the given blend mode.
    Blend(BlendMode),
    /// Applies a 4×5 affine color matrix to the layer's composited RGBA content
    /// (`rgba' = M · rgba + bias`, per-channel, then clamped to `0..1`).
    ///
    /// The 20 entries are the CSS Filter Effects color-matrix folded from an
    /// element's color-op `filter` chain (`grayscale/brightness/contrast/
    /// saturate/sepia/invert/hue-rotate/opacity`): a 4×4 linear part in **row
    /// major** order (`m[row][col] == matrix[row * 4 + col]`, indices `0..16`)
    /// followed by the 4-entry bias (`bias[row] == matrix[16 + row]`). The
    /// coordinator composes the whole affine color-op chain into this single
    /// matrix; the identity chain emits **no** effect (byte-identical no-filter).
    ColorMatrix([f32; 20]),
}

impl LayerEffect {
    /// Creates an opacity effect with the given factor.
    #[must_use]
    pub const fn opacity(factor: f32) -> Self {
        Self::Opacity(factor)
    }

    /// Creates a blur effect with the given radius in logical pixels.
    #[must_use]
    pub const fn blur(radius: f32) -> Self {
        Self::Blur { radius }
    }

    /// Creates a blend effect with the given blend mode.
    #[must_use]
    pub const fn blend(mode: BlendMode) -> Self {
        Self::Blend(mode)
    }

    /// Creates a color-matrix effect from a folded 4×5 affine color matrix (20
    /// entries: a row-major 4×4 linear part followed by a 4-entry bias).
    #[must_use]
    pub const fn color_matrix(matrix: [f32; 20]) -> Self {
        Self::ColorMatrix(matrix)
    }

    /// Returns whether this is an [`LayerEffect::Opacity`] effect.
    #[must_use]
    pub const fn is_opacity(self) -> bool {
        matches!(self, Self::Opacity(_))
    }

    /// Returns whether this is a [`LayerEffect::Blur`] effect.
    #[must_use]
    pub const fn is_blur(self) -> bool {
        matches!(self, Self::Blur { .. })
    }

    /// Returns whether this is a [`LayerEffect::Blend`] effect.
    #[must_use]
    pub const fn is_blend(self) -> bool {
        matches!(self, Self::Blend(_))
    }

    /// Returns whether this is a [`LayerEffect::ColorMatrix`] effect.
    #[must_use]
    pub const fn is_color_matrix(self) -> bool {
        matches!(self, Self::ColorMatrix(_))
    }

    /// Returns the folded 4×5 color matrix if this is a
    /// [`LayerEffect::ColorMatrix`] effect.
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
