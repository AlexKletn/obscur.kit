#![doc = include_str!("effect_report.doc/_module.md")]

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

#[doc = include_str!("effect_report.doc/UNSUPPORTED_EFFECT.md")]
pub const UNSUPPORTED_EFFECT: DiagnosticCode = DiagnosticCode::new("render.effect.unsupported");

#[doc = include_str!("effect_report.doc/EFFECT_FALLBACK.md")]
pub const EFFECT_FALLBACK: DiagnosticCode = DiagnosticCode::new("render.effect.fallback");

#[doc = include_str!("effect_report.doc/EffectFallback.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectFallback {
    effect: SceneEffect,
    reason: Box<str>,
}

impl EffectFallback {
    #[doc = include_str!("effect_report.doc/new.md")]
    #[must_use]
    pub fn new(effect: SceneEffect, reason: impl Into<Box<str>>) -> Self {
        Self {
            effect,
            reason: reason.into(),
        }
    }

    #[doc = include_str!("effect_report.doc/effect.md")]
    #[must_use]
    pub const fn effect(&self) -> SceneEffect {
        self.effect
    }

    #[doc = include_str!("effect_report.doc/reason.md")]
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl std::fmt::Display for EffectFallback {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "degraded effect: {} ({})", self.effect, self.reason)
    }
}

#[doc = include_str!("effect_report.doc/scene_effects.md")]
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

#[doc = include_str!("effect_report.doc/unsupported_effects.md")]
#[must_use]
pub fn unsupported_effects(renderer: &dyn Renderer, list: &DisplayList) -> Vec<SceneEffect> {
    scene_effects(list)
        .into_iter()
        .filter(|effect| renderer.supports_effect(*effect) == SupportLevel::Unavailable)
        .collect()
}

#[doc = include_str!("effect_report.doc/unsupported_effect_error.md")]
#[must_use]
pub fn unsupported_effect_error(effect: SceneEffect) -> RendererError {
    RendererError::UnsupportedEffect(effect.name().into())
}

#[doc = include_str!("effect_report.doc/text_run_font_handles.md")]
#[must_use]
pub fn text_run_font_handles(list: &DisplayList) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    for command in list.commands() {
        if let PaintCommand::DrawText { run, .. } = command
            && !run.glyphs().is_empty()
            && !found.contains(&run.font())
        {
            found.push(run.font());
        }
    }
    found
}

#[doc = include_str!("effect_report.doc/EffectFallbackReporter.md")]
#[derive(Debug, Default, Clone)]
pub struct EffectFallbackReporter {
    reported: Vec<SceneEffect>,
    degraded: Vec<EffectFallback>,
}

impl EffectFallbackReporter {
    #[doc = include_str!("effect_report.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reported: Vec::new(),
            degraded: Vec::new(),
        }
    }

    #[doc = include_str!("effect_report.doc/report.md")]
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

    #[doc = include_str!("effect_report.doc/report_fallbacks.md")]
    pub fn report_fallbacks(
        &mut self,
        fallbacks: &[EffectFallback],
        sink: &dyn DiagnosticSink,
    ) -> Vec<EffectFallback> {
        let mut raised = Vec::new();
        for fallback in fallbacks {
            if self.degraded.contains(fallback) {
                continue;
            }
            self.degraded.push(fallback.clone());
            raised.push(fallback.clone());
            let mut diagnostic = Diagnostic::new(
                EFFECT_FALLBACK,
                DiagnosticSeverity::Warning,
                DiagnosticSubsystem::Render,
                fallback.to_string(),
            );
            diagnostic.recovery = RecoveryAction::Fallback;
            sink.emit(&diagnostic);
        }
        raised
    }

    #[doc = include_str!("effect_report.doc/reported.md")]
    #[must_use]
    pub fn reported(&self) -> &[SceneEffect] {
        &self.reported
    }

    #[doc = include_str!("effect_report.doc/degraded.md")]
    #[must_use]
    pub fn degraded(&self) -> &[EffectFallback] {
        &self.degraded
    }
}

#[doc = include_str!("effect_report.doc/clip_effect.md")]
const fn clip_effect(clip: &Clip) -> SceneEffect {
    match clip {
        Clip::Shape(_) => SceneEffect::Clip,
        Clip::Mask(_) => SceneEffect::Mask,
    }
}

#[doc = include_str!("effect_report.doc/layer_effects.md")]
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
    fn text_run_font_handles_are_deduplicated_and_skip_empty_runs() {
        use obscur_scene::text::{GlyphRun, PositionedGlyph};
        let glyph = || PositionedGlyph::new(1, 0, 0.0, 12.0, 8.0);
        let run = |handle: u32| GlyphRun::new(vec![glyph()], 12.0, "a").with_font(handle);
        let text = |run: GlyphRun| PaintCommand::DrawText {
            origin: Point::new(0.0, 0.0),
            run,
            brush: Brush::Solid(SolidBrush::new(Color::BLACK)),
        };
        let list = DisplayList::from_commands(vec![
            text(run(3)),
            text(run(1)),
            // A repeat of a handle already seen is one demand, not two.
            text(run(3)),
            // An empty run draws nothing with or without its font: not a demand.
            text(GlyphRun::new(Vec::new(), 12.0, "").with_font(9)),
        ]);
        assert_eq!(text_run_font_handles(&list), vec![3, 1]);
        // A scene with no text asks for no font at all.
        assert_eq!(text_run_font_handles(&DisplayList::new()), Vec::new());
    }

    #[test]
    fn fallbacks_are_reported_once_per_effect_and_reason() {
        let sink = RecordingSink::default();
        let mut reporter = EffectFallbackReporter::new();
        let missing_3 = EffectFallback::new(SceneEffect::Text, "no font for handle 3");
        let missing_7 = EffectFallback::new(SceneEffect::Text, "no font for handle 7");

        let raised = reporter.report_fallbacks(&[missing_3.clone(), missing_7.clone()], &sink);
        assert_eq!(raised, vec![missing_3.clone(), missing_7.clone()]);
        assert_eq!(
            *sink.messages.borrow(),
            vec![
                "degraded effect: text (no font for handle 3)".to_string(),
                "degraded effect: text (no font for handle 7)".to_string(),
            ]
        );

        // Same effect, same reason, next frame: already known, so it stays quiet.
        assert!(
            reporter
                .report_fallbacks(std::slice::from_ref(&missing_3), &sink)
                .is_empty()
        );
        assert_eq!(sink.messages.borrow().len(), 2);
        assert_eq!(reporter.degraded(), [missing_3, missing_7]);
    }

    #[test]
    fn a_fallback_and_an_unsupported_effect_do_not_shadow_each_other() {
        // The two halves of §5.2 dedupe independently: a `Text` fallback must not be
        // silenced by an unrelated `Text` entry in the unsupported set, and vice
        // versa. They are different claims — "will not draw it at all" versus "drew
        // it, degraded".
        let sink = RecordingSink::default();
        let mut reporter = EffectFallbackReporter::new();
        reporter.report(&[SceneEffect::Text], &sink);
        let raised = reporter.report_fallbacks(
            &[EffectFallback::new(
                SceneEffect::Text,
                "no font for handle 1",
            )],
            &sink,
        );
        assert_eq!(raised.len(), 1);
        assert_eq!(sink.messages.borrow().len(), 2);
    }

    #[test]
    fn fallback_diagnostic_is_a_render_warning_with_a_fallback_recovery() {
        #[derive(Default)]
        struct Capture(RefCell<Vec<Diagnostic>>);
        impl DiagnosticSink for Capture {
            fn emit(&self, diagnostic: &Diagnostic) {
                self.0.borrow_mut().push(diagnostic.clone());
            }
        }
        let sink = Capture::default();
        EffectFallbackReporter::new().report_fallbacks(
            &[EffectFallback::new(SceneEffect::Text, "the run is skipped")],
            &sink,
        );
        let captured = sink.0.borrow();
        let diagnostic = captured.first().expect("one diagnostic");
        assert_eq!(diagnostic.code, EFFECT_FALLBACK);
        assert_ne!(
            diagnostic.code, UNSUPPORTED_EFFECT,
            "a host must be able to route the two apart"
        );
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(diagnostic.subsystem, DiagnosticSubsystem::Render);
        assert_eq!(diagnostic.recovery, RecoveryAction::Fallback);
        assert_eq!(
            diagnostic.message,
            "degraded effect: text (the run is skipped)"
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
