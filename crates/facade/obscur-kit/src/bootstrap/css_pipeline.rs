#![doc = include_str!("css_pipeline.doc/_module.md")]

use obscur_css::cache::StylesheetCache;
use obscur_css::compiler::{CompiledStylesheet, StylesheetCompiler};
use obscur_css::error::{
    CssParseError, CssParseErrorKind, UnsupportedCssFeature, UnsupportedFeatureKind,
};
use obscur_css::source::CssSource;
use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem, RecoveryAction,
};
use obscur_runtime::UiRuntime;
use obscur_runtime::orchestration::{CssKeyframeRule, CssMotionRules, CssTransitionRule};
use obscur_style::matching::Stylesheet;

#[doc = include_str!("css_pipeline.doc/CssPipeline.md")]
#[derive(Debug, Default)]
pub struct CssPipeline {
    #[doc = include_str!("css_pipeline.doc/cache.md")]
    cache: StylesheetCache,
    #[doc = include_str!("css_pipeline.doc/sheets.md")]
    sheets: Vec<Stylesheet>,
    #[doc = include_str!("css_pipeline.doc/motion.md")]
    motion: CssMotionRules,
    #[doc = include_str!("css_pipeline.doc/errors.md")]
    errors: Vec<CssParseError>,
    #[doc = include_str!("css_pipeline.doc/unsupported.md")]
    unsupported: Vec<UnsupportedCssFeature>,
}

impl CssPipeline {
    #[doc = include_str!("css_pipeline.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("css_pipeline.doc/install.md")]
    pub fn install(&mut self, runtime: &mut UiRuntime, sources: &[CssSource]) {
        self.sheets.clear();
        self.errors.clear();
        self.unsupported.clear();
        let mut transitions = Vec::new();
        let mut keyframes = Vec::new();

        for source in sources {
            let compiled = StylesheetCompiler::new().compile(self.cache.parse(source));
            collect_motion(&compiled, &mut transitions, &mut keyframes);
            self.sheets.push(compiled.matchable);
            self.errors.extend(compiled.errors);
            self.unsupported.extend(compiled.unsupported);
        }

        for error in &self.errors {
            runtime.report_diagnostic(&diagnostic_for_error(error));
        }
        for feature in &self.unsupported {
            runtime.report_diagnostic(&diagnostic_for_unsupported(feature));
        }

        self.motion = CssMotionRules::from_parts(transitions, keyframes);
        runtime.set_stylesheets(self.sheets.clone());
        runtime.set_css_motion(self.motion.clone());
    }

    #[doc = include_str!("css_pipeline.doc/sheets_2.md")]
    #[must_use]
    pub fn sheets(&self) -> &[Stylesheet] {
        &self.sheets
    }

    #[doc = include_str!("css_pipeline.doc/motion_2.md")]
    #[must_use]
    pub fn motion(&self) -> &CssMotionRules {
        &self.motion
    }

    #[doc = include_str!("css_pipeline.doc/errors_2.md")]
    #[must_use]
    pub fn errors(&self) -> &[CssParseError] {
        &self.errors
    }

    #[doc = include_str!("css_pipeline.doc/unsupported_2.md")]
    #[must_use]
    pub fn unsupported(&self) -> &[UnsupportedCssFeature] {
        &self.unsupported
    }

    #[doc = include_str!("css_pipeline.doc/cached_sources.md")]
    #[must_use]
    pub fn cached_sources(&self) -> usize {
        self.cache.len()
    }

    #[doc = include_str!("css_pipeline.doc/invalidate_cache.md")]
    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
    }
}

#[doc = include_str!("css_pipeline.doc/diagnostic_for_error.md")]
fn diagnostic_for_error(error: &CssParseError) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new(code_for_parse_error(error.kind())),
        DiagnosticSeverity::Error,
        DiagnosticSubsystem::Style,
        error.to_string(),
    )
    // The offending declaration/rule is dropped and the rest of the sheet is
    // kept — a localized failure, not a torn-down sheet (spec §1.6).
    .with_recovery(RecoveryAction::Fallback)
}

#[doc = include_str!("css_pipeline.doc/diagnostic_for_unsupported.md")]
fn diagnostic_for_unsupported(feature: &UnsupportedCssFeature) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new(code_for_unsupported(feature.kind())),
        DiagnosticSeverity::Warning,
        DiagnosticSubsystem::Style,
        feature.to_string(),
    )
    .with_recovery(RecoveryAction::Fallback)
}

#[doc = include_str!("css_pipeline.doc/code_for_parse_error.md")]
const fn code_for_parse_error(kind: CssParseErrorKind) -> &'static str {
    match kind {
        CssParseErrorKind::UnexpectedToken => "css.unexpected_token",
        CssParseErrorKind::UnexpectedEof => "css.unexpected_eof",
        CssParseErrorKind::InvalidSelector => "css.invalid_selector",
        CssParseErrorKind::InvalidValue => "css.invalid_value",
        CssParseErrorKind::ExpectedColon => "css.expected_colon",
        CssParseErrorKind::ExpectedSemicolon => "css.expected_semicolon",
        CssParseErrorKind::UnterminatedBlock => "css.unterminated_block",
        CssParseErrorKind::UnterminatedString => "css.unterminated_string",
        CssParseErrorKind::InvalidAtRule => "css.invalid_at_rule",
    }
}

#[doc = include_str!("css_pipeline.doc/code_for_unsupported.md")]
const fn code_for_unsupported(kind: UnsupportedFeatureKind) -> &'static str {
    match kind {
        UnsupportedFeatureKind::AtRule => "css.unsupported_at_rule",
        UnsupportedFeatureKind::Property => "css.unsupported_property",
        UnsupportedFeatureKind::Selector => "css.unsupported_selector",
        UnsupportedFeatureKind::Function => "css.unsupported_function",
        UnsupportedFeatureKind::Unit => "css.unsupported_unit",
        UnsupportedFeatureKind::PseudoElement => "css.unsupported_pseudo_element",
    }
}

#[doc = include_str!("css_pipeline.doc/collect_motion.md")]
fn collect_motion(
    compiled: &CompiledStylesheet,
    transitions: &mut Vec<CssTransitionRule>,
    keyframes: &mut Vec<CssKeyframeRule>,
) {
    for rule in &compiled.motion_rules {
        for transition in &rule.transitions {
            transitions.push(
                CssTransitionRule::new(
                    rule.selectors.clone(),
                    transition.property,
                    transition.duration_secs,
                    transition.easing,
                )
                .with_delay(transition.delay_secs),
            );
        }
        for animation in &rule.animations {
            keyframes.push(
                CssKeyframeRule::new(
                    rule.selectors.clone(),
                    animation.tracks.clone(),
                    animation.duration_secs,
                )
                .with_playback(
                    animation.delay_secs,
                    animation.repeat,
                    animation.direction,
                    animation.alternates,
                    animation.fill,
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CssPipeline;
    use obscur_css::source::{CssSource, SourceId};
    use obscur_geometry::Size;
    use obscur_identity::RuntimeId;
    use obscur_runtime::UiRuntime;

    fn runtime() -> UiRuntime {
        UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 100.0))
    }

    #[test]
    fn install_carries_both_halves_of_the_sheet() {
        let mut runtime = runtime();
        let mut pipeline = CssPipeline::new();
        pipeline.install(
            &mut runtime,
            &[CssSource::anonymous(
                ".box { width: 10px; transition: width 1s linear; }",
            )],
        );
        assert_eq!(pipeline.sheets().len(), 1, "the matchable rules");
        assert_eq!(
            pipeline.motion().transitions().len(),
            1,
            "and the motion the runtime would otherwise never see"
        );
    }

    #[test]
    fn an_anonymous_source_is_never_cached() {
        let mut runtime = runtime();
        let mut pipeline = CssPipeline::new();
        pipeline.install(&mut runtime, &[CssSource::anonymous(".a {}")]);
        assert_eq!(pipeline.cached_sources(), 0);
    }

    #[test]
    fn a_reinstall_replaces_rather_than_accumulates() {
        let mut runtime = runtime();
        let mut pipeline = CssPipeline::new();
        let source = |text: &str| CssSource::new(SourceId::from_raw(1), "app.css", text);
        pipeline.install(&mut runtime, &[source(".a {} .b {}"), source(".c {}")]);
        assert_eq!(pipeline.sheets().len(), 2);
        pipeline.install(&mut runtime, &[source(".a {}")]);
        assert_eq!(pipeline.sheets().len(), 1, "the previous set is dropped");
    }

    #[test]
    fn unsupported_features_are_reported_not_fatal() {
        let mut runtime = runtime();
        let mut pipeline = CssPipeline::new();
        pipeline.install(&mut runtime, &[CssSource::anonymous("a { float: left; }")]);
        assert!(pipeline.errors().is_empty(), "syntactically valid");
        assert!(!pipeline.unsupported().is_empty(), "`float` is banned (§9)");
    }
}
