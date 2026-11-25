#![doc = include_str!("../../docs/api/renderer/effect_report.doc/_module.md")]

use crate::renderer::{EffectFallback, RendererError, SceneEffect};
use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSink, DiagnosticSubsystem,
    RecoveryAction,
};

/// The stable code of the diagnostic raised when a backend will not reproduce an
/// effect a scene demands (spec §5.2).
pub const UNSUPPORTED_EFFECT: DiagnosticCode = DiagnosticCode::new("render.effect.unsupported");

#[doc = include_str!("../../docs/api/renderer/effect_report.doc/EFFECT_FALLBACK.md")]
pub const EFFECT_FALLBACK: DiagnosticCode = DiagnosticCode::new("render.effect.fallback");

/// The typed error naming an effect a backend will not reproduce — the explicit
/// §5.2 fallback diagnostic in its canonical form.
#[must_use]
pub fn unsupported_effect_error(effect: SceneEffect) -> RendererError {
    RendererError::UnsupportedEffect(effect.name().into())
}

#[doc = include_str!("../../docs/api/renderer/effect_report.doc/EffectFallbackReporter.md")]
#[derive(Debug, Default, Clone)]
pub struct EffectFallbackReporter {
    reported: Vec<SceneEffect>,
    degraded: Vec<EffectFallback>,
}

impl EffectFallbackReporter {
    /// A reporter that has raised nothing yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reported: Vec::new(),
            degraded: Vec::new(),
        }
    }

    #[doc = include_str!("../../docs/api/renderer/effect_report.doc/report.md")]
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

    #[doc = include_str!("../../docs/api/renderer/effect_report.doc/report_fallbacks.md")]
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

    /// The effects this reporter has already raised.
    #[must_use]
    pub fn reported(&self) -> &[SceneEffect] {
        &self.reported
    }

    /// The fallbacks this reporter has already raised.
    #[must_use]
    pub fn degraded(&self) -> &[EffectFallback] {
        &self.degraded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
