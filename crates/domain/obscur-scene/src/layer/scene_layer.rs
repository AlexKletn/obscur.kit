//! A compositing layer descriptor.

use obscur_geometry::AffineTransform;

use crate::clip::Clip;
use crate::paint::BlendMode;

use super::{CompositionMode, LayerEffect};

/// A compositing layer descriptor: the parameters under which a group of
/// content is composited (spec §5.1).
///
/// A `SceneLayer` carries **no content**. The content it applies to is a display
/// list supplied by the command model (a later sub-step); this type is only the
/// compositing state pushed around that content — its transform, optional clip,
/// opacity, blend mode, Porter-Duff operator, and any additional effects. It is
/// backend-independent and references no GPU or renderer types.
///
/// A layer whose parameters are all neutral needs no offscreen pass and can be
/// flattened by the renderer; see [`SceneLayer::is_trivial`].
#[derive(Debug, Clone, PartialEq)]
pub struct SceneLayer {
    transform: AffineTransform,
    clip: Option<Clip>,
    opacity: f32,
    blend_mode: BlendMode,
    composition: CompositionMode,
    effects: Vec<LayerEffect>,
}

impl SceneLayer {
    /// Creates a neutral layer: identity transform, no clip, opacity `1.0`,
    /// [`BlendMode::Normal`], [`CompositionMode::SrcOver`], and no effects.
    #[must_use]
    pub fn new() -> Self {
        Self {
            transform: AffineTransform::IDENTITY,
            clip: None,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            composition: CompositionMode::SrcOver,
            effects: Vec::new(),
        }
    }

    /// Returns the layer with its local transform replaced.
    #[must_use]
    pub fn with_transform(mut self, transform: AffineTransform) -> Self {
        self.transform = transform;
        self
    }

    /// Returns the layer with a clip applied to its content.
    #[must_use]
    pub fn with_clip(mut self, clip: Clip) -> Self {
        self.clip = Some(clip);
        self
    }

    /// Returns the layer with its opacity replaced (nominally `0.0..=1.0`).
    #[must_use]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Returns the layer with its blend mode replaced.
    #[must_use]
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Returns the layer with its composition operator replaced.
    #[must_use]
    pub fn with_composition(mut self, composition: CompositionMode) -> Self {
        self.composition = composition;
        self
    }

    /// Appends an additional effect (e.g. a blur) to the layer.
    pub fn add_effect(&mut self, effect: LayerEffect) {
        self.effects.push(effect);
    }

    /// The layer's local transform.
    #[must_use]
    pub fn transform(&self) -> AffineTransform {
        self.transform
    }

    /// The layer's optional clip.
    #[must_use]
    pub fn clip(&self) -> Option<&Clip> {
        self.clip.as_ref()
    }

    /// The layer's opacity.
    #[must_use]
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// The layer's blend mode.
    #[must_use]
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    /// The layer's composition operator.
    #[must_use]
    pub fn composition(&self) -> CompositionMode {
        self.composition
    }

    /// The layer's additional effects, in application order.
    #[must_use]
    pub fn effects(&self) -> &[LayerEffect] {
        &self.effects
    }

    /// Returns whether the layer is trivial: identity transform, no clip,
    /// opacity `>= 1.0`, [`BlendMode::Normal`], [`CompositionMode::SrcOver`],
    /// and no effects.
    ///
    /// A trivial layer changes nothing about how its content composites, so the
    /// renderer can flatten it away without an offscreen pass.
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        self.transform == AffineTransform::IDENTITY
            && self.clip.is_none()
            && self.opacity >= 1.0
            && self.blend_mode.is_normal()
            && self.composition.is_src_over()
            && self.effects.is_empty()
    }
}

impl Default for SceneLayer {
    /// The default layer is the neutral layer produced by [`SceneLayer::new`].
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clip::Clip;
    use obscur_geometry::{Point, Rect, Size};

    #[test]
    fn default_is_neutral_and_trivial() {
        let layer = SceneLayer::default();
        assert_eq!(layer.transform(), AffineTransform::IDENTITY);
        assert!(layer.clip().is_none());
        assert_eq!(layer.opacity(), 1.0);
        assert_eq!(layer.blend_mode(), BlendMode::Normal);
        assert_eq!(layer.composition(), CompositionMode::SrcOver);
        assert!(layer.effects().is_empty());
        assert!(layer.is_trivial());
    }

    #[test]
    fn transform_makes_non_trivial() {
        let layer = SceneLayer::new().with_transform(AffineTransform::translate(1.0, 0.0));
        assert!(!layer.is_trivial());
    }

    #[test]
    fn clip_makes_non_trivial() {
        let clip = Clip::rect(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)));
        let layer = SceneLayer::new().with_clip(clip);
        assert!(layer.clip().is_some());
        assert!(!layer.is_trivial());
    }

    #[test]
    fn reduced_opacity_makes_non_trivial() {
        assert!(!SceneLayer::new().with_opacity(0.5).is_trivial());
    }

    #[test]
    fn opacity_above_one_stays_trivial() {
        assert!(SceneLayer::new().with_opacity(1.5).is_trivial());
    }

    #[test]
    fn blend_mode_makes_non_trivial() {
        assert!(
            !SceneLayer::new()
                .with_blend_mode(BlendMode::Multiply)
                .is_trivial()
        );
    }

    #[test]
    fn composition_makes_non_trivial() {
        assert!(
            !SceneLayer::new()
                .with_composition(CompositionMode::Clear)
                .is_trivial()
        );
    }

    #[test]
    fn effect_makes_non_trivial() {
        let mut layer = SceneLayer::new();
        layer.add_effect(LayerEffect::blur(2.0));
        assert_eq!(layer.effects().len(), 1);
        assert!(!layer.is_trivial());
    }

    #[test]
    fn builders_are_chainable() {
        let layer = SceneLayer::new()
            .with_opacity(0.25)
            .with_blend_mode(BlendMode::Screen)
            .with_composition(CompositionMode::Src);
        assert_eq!(layer.opacity(), 0.25);
        assert_eq!(layer.blend_mode(), BlendMode::Screen);
        assert_eq!(layer.composition(), CompositionMode::Src);
    }
}
