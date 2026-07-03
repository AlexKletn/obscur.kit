//! An effect applied to a layer's composited content.

use crate::paint::BlendMode;

/// An effect applied to a layer's composited content (spec §5.1).
///
/// A `LayerEffect` is a backend-independent descriptor of a post-composition
/// operation: uniform opacity, a Gaussian-style blur, or a blend mode against
/// the backdrop. It references no GPU or renderer types. Shadows and color
/// filters are later additions.
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
}
