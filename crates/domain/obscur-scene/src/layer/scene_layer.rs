//! A compositing layer descriptor.

use obscur_geometry::AffineTransform;

use crate::clip::Clip;
use crate::paint::BlendMode;

use super::{CompositionMode, LayerEffect};

#[doc = include_str!("../../docs/api/layer/scene_layer.doc/SceneLayer.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct SceneLayer {
    transform: AffineTransform,
    clip: Option<Clip>,
    opacity: f32,
    isolated: bool,
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
            isolated: false,
            blend_mode: BlendMode::Normal,
            composition: CompositionMode::SrcOver,
            effects: Vec::new(),
        }
    }

    /// Requires a fresh transparent backdrop for the children of this layer.
    #[must_use]
    pub fn with_isolation(mut self, isolated: bool) -> Self {
        self.isolated = isolated;
        self
    }

    /// Whether the layer requires an independent blending backdrop.
    #[must_use]
    pub const fn is_isolated(&self) -> bool {
        self.isolated
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

    /// The local transform composed with the parent transform for the layer's clip
    /// and drawing commands. The parent transform resumes when the layer closes.
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

    /// The mode used to composite the layer against its backdrop. The last
    /// blend effect takes precedence over the mode set by the builder.
    #[must_use]
    pub fn blend_mode(&self) -> BlendMode {
        self.effects
            .iter()
            .rev()
            .find_map(|effect| match effect {
                LayerEffect::Blend(mode) => Some(*mode),
                _ => None,
            })
            .unwrap_or(self.blend_mode)
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

    #[doc = include_str!("../../docs/api/layer/scene_layer.doc/is_trivial.md")]
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        !self.isolated
            && self.transform == AffineTransform::IDENTITY
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
