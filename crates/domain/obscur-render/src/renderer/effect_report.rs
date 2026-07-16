//! The §5.2 explicit fallback diagnostic: what a scene demands, what a backend
//! will not reproduce, and how that reaches a host.

use obscur_capability::SupportLevel;
use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};
use obscur_scene::clip::Clip;
use obscur_scene::command::{DisplayList, PaintCommand};
use obscur_scene::layer::{LayerEffect, SceneLayer};
use obscur_scene::paint::{BlendMode, Brush, Paint};

use crate::renderer::{Renderer, RendererError, SceneEffect};

/// The stable code of the diagnostic raised when a backend will not reproduce an
/// effect a scene demands (spec §5.2).
pub const UNSUPPORTED_EFFECT: DiagnosticCode = DiagnosticCode::new("render.effect.unsupported");

/// Every [`SceneEffect`] `list` demands, deduplicated, in first-use paint order
/// (spec §5.2).
///
/// This is the one place the display list is read as a set of *effect demands*, so
/// a backend and a host agree on what a scene needs without either re-deriving it.
#[must_use]
pub fn scene_effects(list: &DisplayList) -> Vec<SceneEffect> {
    let mut found: Vec<SceneEffect> = Vec::new();
    let mut push = |effect: SceneEffect| {
        if !found.contains(&effect) {
            found.push(effect);
        }
    };
    for command in list.commands() {
        match command {
            PaintCommand::DrawPath { paint, .. } => {
                let brush = match paint {
                    Paint::Fill(brush) => brush,
                    Paint::Stroke(stroke) => stroke.brush(),
                };
                if !matches!(brush, Brush::Solid(_)) {
                    push(SceneEffect::Gradient);
                }
            }
            PaintCommand::DrawText { .. } => push(SceneEffect::Text),
            PaintCommand::DrawShadow { blur, .. } => {
                push(SceneEffect::Shadow);
                // A zero radius is a hard-edged shadow by definition, so it demands
                // no blur; any positive radius does.
                if *blur > 0.0 {
                    push(SceneEffect::Blur);
                }
            }
            PaintCommand::DrawImage { .. } => push(SceneEffect::Image),
            PaintCommand::BackdropFilter { blur, .. } => {
                // `blur: 0` is documented as no effect at all.
                if *blur > 0.0 {
                    push(SceneEffect::BackdropFilter);
                }
            }
            PaintCommand::PushClip(clip) => push(clip_effect(clip)),
            PaintCommand::PushLayer(layer) => layer_effects(layer, &mut push),
            PaintCommand::PushTransform(_)
            | PaintCommand::PopTransform
            | PaintCommand::PopClip
            | PaintCommand::PopLayer => {}
        }
    }
    found
}

/// The effects `list` demands that `renderer` reports it will **not** reproduce
/// ([`SupportLevel::Unavailable`]), in first-use paint order (spec §5.2).
///
/// Only `Unavailable` is collected: it is the level at which the effect does not
/// happen at all. A [`SupportLevel::Limited`] or [`SupportLevel::Emulated`] effect
/// *does* reach the frame — as the approximation documented on the backend's
/// [`Renderer::supports_effect`] — and is queried there rather than raised per
/// scene.
#[must_use]
pub fn unsupported_effects(renderer: &dyn Renderer, list: &DisplayList) -> Vec<SceneEffect> {
    scene_effects(list)
        .into_iter()
        .filter(|effect| renderer.supports_effect(*effect) == SupportLevel::Unavailable)
        .collect()
}

/// The typed error naming an effect a backend will not reproduce — the explicit
/// §5.2 fallback diagnostic in its canonical form.
#[must_use]
pub fn unsupported_effect_error(effect: SceneEffect) -> RendererError {
    RendererError::UnsupportedEffect(effect.name().into())
}

/// Emits the §5.2 explicit fallback diagnostic for the effects a backend will not
/// reproduce, once per effect.
///
/// A backend owns one of these and calls [`report`](Self::report) from its render
/// path with the [`unsupported_effects`] of the scene it is about to draw. Whether
/// a backend reproduces an effect is a property of the **backend**, not of the
/// frame, so each effect is reported the first time a scene demands it and never
/// again: a host learns "this backend will not draw your gradients" once, instead
/// of sixty times a second for as long as the gradient is on screen.
///
/// The diagnostic is [`Warning`](DiagnosticSeverity::Warning) with
/// [`RecoveryAction::Fallback`]: the frame is still drawn, with the backend's
/// documented approximation in place of the effect.
#[derive(Debug, Default, Clone)]
pub struct EffectFallbackReporter {
    reported: Vec<SceneEffect>,
}

impl EffectFallbackReporter {
    /// A reporter that has raised nothing yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reported: Vec::new(),
        }
    }

    /// Emits one diagnostic per effect in `unsupported` not yet reported, and
    /// returns them.
    ///
    /// Pass the result of [`unsupported_effects`] for the scene about to be drawn.
    pub fn report(
        &mut self,
        unsupported: &[SceneEffect],
        sink: &dyn DiagnosticSink,
    ) -> Vec<SceneEffect> {
        let mut raised = Vec::new();
        for effect in unsupported {
            if self.reported.contains(effect) {
                continue;
            }
            self.reported.push(*effect);
            raised.push(*effect);
            // The error type IS the message: `UnsupportedEffect` is the canonical
            // rendering of "this backend will not reproduce X", and building it here
            // keeps the diagnostic and the port's error variant from drifting apart.
            let error = unsupported_effect_error(*effect);
            let mut diagnostic = Diagnostic::new(
                UNSUPPORTED_EFFECT,
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Render,
                error.to_string(),
            );
            diagnostic.recovery = RecoveryAction::Fallback;
            sink.emit(&diagnostic);
        }
        raised
    }

    /// The effects this reporter has already raised.
    #[must_use]
    pub fn reported(&self) -> &[SceneEffect] {
        &self.reported
    }
}

/// The effect a clip demands: a shape clip confines by geometry, a mask clip by
/// coverage read from separate source content.
const fn clip_effect(clip: &Clip) -> SceneEffect {
    match clip {
        Clip::Shape(_) => SceneEffect::Clip,
        Clip::Mask(_) => SceneEffect::Mask,
    }
}

/// The effects a `PushLayer` demands, pushed through `push`.
fn layer_effects(layer: &SceneLayer, push: &mut impl FnMut(SceneEffect)) {
    if layer.blend_mode() != BlendMode::Normal {
        push(SceneEffect::BlendMode);
    }
    if let Some(clip) = layer.clip() {
        push(clip_effect(clip));
    }
    for effect in layer.effects() {
        match effect {
            LayerEffect::Blur { radius } if *radius > 0.0 => push(SceneEffect::Blur),
            LayerEffect::ColorMatrix(_) => push(SceneEffect::ColorMatrix),
            LayerEffect::Blend(mode) if *mode != BlendMode::Normal => push(SceneEffect::BlendMode),
            // A zero-radius blur and a `Normal` blend ask for nothing. Group opacity
            // reaches a backend as the layer's `opacity` FIELD (which both in-tree
            // backends honor), never as this effect; a `LayerEffect::Opacity` would
            // be dropped by both, and it has no variant in the vocabulary, so it is
            // not diagnosed here.
            LayerEffect::Blur { .. } | LayerEffect::Blend(_) | LayerEffect::Opacity(_) => {}
        }
    }
    // The layer's own `transform` and Porter-Duff `composition` are compositing
    // parameters, not effects: they are outside this vocabulary (see `SceneEffect`).
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{HeadlessRenderer, RendererId};
    use obscur_geometry::{CornerRadii, Point, Rect, Size};
    use obscur_scene::clip::Mask;
    use obscur_scene::color::Color;
    use obscur_scene::paint::{GradientStop, LinearGradient, SolidBrush};
    use obscur_scene::path::Path;
    use std::cell::RefCell;

    #[derive(Default)]
    struct RecordingSink {
        messages: RefCell<Vec<String>>,
    }

    impl DiagnosticSink for RecordingSink {
        fn emit(&self, diagnostic: &Diagnostic) {
            self.messages.borrow_mut().push(diagnostic.message.clone());
        }
    }

    fn a_rect() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(10.0, 10.0))
    }

    fn gradient_brush() -> Brush {
        Brush::Linear(LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            vec![
                GradientStop::new(0.0, Color::BLACK),
                GradientStop::new(1.0, Color::WHITE),
            ],
        ))
    }

    fn shadow(blur: f32) -> PaintCommand {
        PaintCommand::DrawShadow {
            rect: a_rect(),
            radii: CornerRadii::uniform(0.0),
            color: Color::BLACK,
            blur,
            spread: 0.0,
            offset: (1.0, 1.0),
            inset: false,
        }
    }

    #[test]
    fn solid_fill_demands_no_effect() {
        let list = DisplayList::from_commands(vec![PaintCommand::DrawPath {
            path: Path::new(),
            paint: Paint::Fill(Brush::Solid(SolidBrush::new(Color::BLACK))),
        }]);
        assert_eq!(scene_effects(&list), Vec::new());
    }

    #[test]
    fn gradient_fill_and_stroke_both_demand_gradient() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::DrawPath {
                path: Path::new(),
                paint: Paint::Fill(gradient_brush()),
            },
            PaintCommand::DrawPath {
                path: Path::new(),
                paint: Paint::Stroke(obscur_scene::paint::Stroke::new(gradient_brush(), 1.0)),
            },
        ]);
        // Deduplicated: two gradient draws are one demand.
        assert_eq!(scene_effects(&list), vec![SceneEffect::Gradient]);
    }

    #[test]
    fn shadow_demands_blur_only_when_its_radius_is_positive() {
        let hard = DisplayList::from_commands(vec![shadow(0.0)]);
        assert_eq!(scene_effects(&hard), vec![SceneEffect::Shadow]);
        let soft = DisplayList::from_commands(vec![shadow(4.0)]);
        assert_eq!(
            scene_effects(&soft),
            vec![SceneEffect::Shadow, SceneEffect::Blur]
        );
    }

    #[test]
    fn clip_and_mask_are_distinct_demands() {
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushClip(Clip::rect(a_rect())),
            PaintCommand::PopClip,
            PaintCommand::PushClip(Clip::mask(Mask::alpha())),
            PaintCommand::PopClip,
        ]);
        assert_eq!(
            scene_effects(&list),
            vec![SceneEffect::Clip, SceneEffect::Mask]
        );
    }

    #[test]
    fn layer_demands_its_blend_clip_and_effects() {
        let mut layer = SceneLayer::new()
            .with_blend_mode(BlendMode::Multiply)
            .with_clip(Clip::rect(a_rect()));
        layer.add_effect(LayerEffect::blur(3.0));
        layer.add_effect(LayerEffect::color_matrix([0.0; 20]));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::PopLayer,
        ]);
        assert_eq!(
            scene_effects(&list),
            vec![
                SceneEffect::BlendMode,
                SceneEffect::Clip,
                SceneEffect::Blur,
                SceneEffect::ColorMatrix,
            ]
        );
    }

    #[test]
    fn group_opacity_is_not_in_the_vocabulary() {
        // The layer's opacity FIELD is honored by both backends and demands nothing;
        // a `LayerEffect::Opacity` is outside the vocabulary and is not diagnosed.
        let mut layer = SceneLayer::new().with_opacity(0.5);
        layer.add_effect(LayerEffect::opacity(0.5));
        let list = DisplayList::from_commands(vec![
            PaintCommand::PushLayer(layer),
            PaintCommand::PopLayer,
        ]);
        assert_eq!(scene_effects(&list), Vec::new());
    }

    #[test]
    fn unsupported_effects_filters_by_the_backend_table() {
        // `HeadlessRenderer` rasterizes nothing, so every demand is unavailable.
        let renderer = HeadlessRenderer::new(RendererId::from_raw(1));
        let list = DisplayList::from_commands(vec![shadow(2.0)]);
        assert_eq!(
            unsupported_effects(&renderer, &list),
            vec![SceneEffect::Shadow, SceneEffect::Blur]
        );
        // An empty scene demands nothing, so nothing is unsupported.
        assert_eq!(
            unsupported_effects(&renderer, &DisplayList::new()),
            Vec::new()
        );
    }

    #[test]
    fn reporter_emits_the_error_text_once_per_effect() {
        let sink = RecordingSink::default();
        let mut reporter = EffectFallbackReporter::new();
        let raised = reporter.report(&[SceneEffect::Blur, SceneEffect::Gradient], &sink);
        assert_eq!(raised, vec![SceneEffect::Blur, SceneEffect::Gradient]);
        assert_eq!(
            *sink.messages.borrow(),
            vec![
                "unsupported effect: blur".to_string(),
                "unsupported effect: gradient".to_string(),
            ]
        );
        // A second frame demanding the same effects raises nothing further: the
        // answer is a property of the backend, not of the frame.
        assert!(
            reporter
                .report(&[SceneEffect::Blur, SceneEffect::Gradient], &sink)
                .is_empty()
        );
        assert_eq!(sink.messages.borrow().len(), 2);
        assert_eq!(
            reporter.reported(),
            [SceneEffect::Blur, SceneEffect::Gradient]
        );
    }

    #[test]
    fn reported_diagnostic_is_a_render_warning_with_a_fallback_recovery() {
        #[derive(Default)]
        struct Capture(RefCell<Vec<Diagnostic>>);
        impl DiagnosticSink for Capture {
            fn emit(&self, diagnostic: &Diagnostic) {
                self.0.borrow_mut().push(diagnostic.clone());
            }
        }
        let sink = Capture::default();
        EffectFallbackReporter::new().report(&[SceneEffect::BackdropFilter], &sink);
        let captured = sink.0.borrow();
        let diagnostic = captured.first().expect("one diagnostic");
        assert_eq!(diagnostic.code, UNSUPPORTED_EFFECT);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.subsystem, DiagnosticSubsystem::Render);
        assert_eq!(diagnostic.recovery, RecoveryAction::Fallback);
        assert_eq!(diagnostic.message, "unsupported effect: backdrop-filter");
    }
}
