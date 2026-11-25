#![doc = include_str!("../../docs/api/bootstrap/css_pipeline.doc/_module.md")]

use std::rc::Rc;

use obscur_css::cache::StylesheetCache;
use obscur_css::compiler::{
    CompiledStylesheet, CustomValueCache, StylesheetCompiler, compile_inline_style,
};
use obscur_css::error::{
    CssParseError, CssParseErrorKind, UnsupportedCssFeature, UnsupportedFeatureKind,
};
use obscur_css::source::CssSource;
use obscur_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem, RecoveryAction,
};
use obscur_runtime::UiRuntime;
use obscur_runtime::orchestration::{
    CssKeyframeRule, CssMotionRules, CssTransitionRule, InlineStyleOutcome,
};
use obscur_style::matching::Stylesheet;

#[doc = include_str!("../../docs/api/bootstrap/css_pipeline.doc/CssPipeline.md")]
#[derive(Debug, Default)]
pub struct CssPipeline {
    /// The §4.12 parse cache, keyed by `(SourceId, SourceVersion)`. Retained
    /// across installs: on a reload of N sources where one changed, only that
    /// one is re-parsed.
    cache: StylesheetCache,
    /// The matchable sheets from the last [`install`](Self::install), in source
    /// order — the same slice handed to the runtime.
    sheets: Vec<Stylesheet>,
    /// The motion rules from the last [`install`](Self::install).
    motion: CssMotionRules,
    /// Syntax errors from the last [`install`](Self::install), across all sources.
    errors: Vec<CssParseError>,
    /// Recognised-but-unsupported features from the last
    /// [`install`](Self::install), across all sources.
    unsupported: Vec<UnsupportedCssFeature>,
}

impl CssPipeline {
    /// Creates an empty pipeline with a cold cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/bootstrap/css_pipeline.doc/install.md")]
    pub fn install(&mut self, runtime: &mut UiRuntime, sources: &[CssSource]) {
        self.sheets.clear();
        self.errors.clear();
        self.unsupported.clear();
        let mut transitions = Vec::new();
        let mut keyframes = Vec::new();

        for source in sources {
            let compiled = StylesheetCompiler::new().compile(self.cache.parse(source));
            collect_motion(&compiled, &mut transitions, &mut keyframes);
            // `compiled.custom_properties` is deliberately not forwarded: it is a
            // flat per-SHEET list that cannot say which elements a `--name` was
            // declared for, and `:root { --brand: red }` next to
            // `.dark { --brand: blue }` is two answers for two elements. The
            // declarations the runtime actually cascades ride inside
            // `matchable`, on the rule that declared them.
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
        self.install_on(runtime);
    }

    pub(crate) fn install_on(&self, runtime: &mut UiRuntime) {
        runtime.set_stylesheets(self.sheets.clone());
        runtime.set_css_motion(self.motion.clone());
        install_inline_style_compiler(runtime);
        install_custom_value_parser(runtime);
    }

    /// The matchable sheets from the last [`install`](Self::install), for the
    /// render paths that take an explicit `&[Stylesheet]`
    /// ([`render_now`](crate::runtime::UiRuntime::render_now) and friends). The
    /// reactive path reads the copy `install` left on the runtime instead.
    #[must_use]
    pub fn sheets(&self) -> &[Stylesheet] {
        &self.sheets
    }

    /// The CSS-authored motion from the last [`install`](Self::install) — the
    /// same rules the runtime holds.
    #[must_use]
    pub fn motion(&self) -> &CssMotionRules {
        &self.motion
    }

    /// The syntax errors from the last [`install`](Self::install), across all
    /// sources.
    #[must_use]
    pub fn errors(&self) -> &[CssParseError] {
        &self.errors
    }

    /// The recognised-but-unsupported features from the last
    /// [`install`](Self::install), across all sources.
    #[must_use]
    pub fn unsupported(&self) -> &[UnsupportedCssFeature] {
        &self.unsupported
    }

    /// How many identified sources are held in the parse cache. Anonymous sources
    /// are never cached, so they never count here.
    #[must_use]
    pub fn cached_sources(&self) -> usize {
        self.cache.len()
    }

    #[doc = include_str!("../../docs/api/bootstrap/css_pipeline.doc/invalidate_cache.md")]
    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
    }
}

#[doc = include_str!("../../docs/api/bootstrap/css_pipeline.doc/install_inline_style_compiler.md")]
fn install_inline_style_compiler(runtime: &mut UiRuntime) {
    // Guarded, not unconditional: this runs again on every live stylesheet
    // replacement, and re-installing would throw away every block compiled so
    // far. Nothing about an inline block depends on the sheets it competes with,
    // so the old answers are still this compiler's answers.
    if runtime.has_inline_style_compiler() {
        return;
    }
    // The application tier cannot depend on a frontend crate (§1.11), so the
    // grammar reaches the runtime as a function. This is the one place that owns
    // both sides, which is why the `style` attribute works exactly where the CSS
    // pipeline does and is inert where it is not.
    runtime.set_inline_style_compiler(Box::new(|text| {
        let compiled = compile_inline_style(text);
        // Through the same two constructors an authored sheet's faults go
        // through: a misspelled property must not be reported one way in a
        // stylesheet and another way in an attribute.
        let faults = compiled
            .errors
            .iter()
            .map(diagnostic_for_error)
            .chain(compiled.unsupported.iter().map(diagnostic_for_unsupported))
            .collect();
        InlineStyleOutcome::new(compiled.style).with_faults(faults)
    }));
}

#[doc = include_str!("../../docs/api/bootstrap/css_pipeline.doc/install_custom_value_parser.md")]
pub(super) fn install_custom_value_parser(runtime: &mut UiRuntime) {
    if runtime.has_custom_value_parser() {
        return;
    }
    // A cache, not the bare lowering function. The grammar is asked the same
    // question once per `var()` per node per restyle — `.key` alone carries
    // fifteen of them — and the answer to any one of those questions never
    // changes, so running a CSS tokenizer over the same declared text again was
    // the whole of the cost.
    runtime.set_custom_value_parser(Rc::new(CustomValueCache::new()));
}

/// Builds the `Style`-subsystem diagnostic for a CSS syntactic error. The
/// [`stable code`](DiagnosticCode) is derived from the error's *kind*, not its
/// (localizable) message or its source span, so it is identical across locales
/// and call sites (spec §1.6). The human-readable message carries the detail.
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

/// Builds the `Style`-subsystem diagnostic for a recognised-but-unsupported CSS
/// construct. Unsupported features degrade gracefully (§4.13), so this is a
/// warning, not an error; its stable code is derived from the feature's kind.
fn diagnostic_for_unsupported(feature: &UnsupportedCssFeature) -> Diagnostic {
    // Another vendor's property is INFO, not a warning. A stylesheet shared
    // with a browser build carries `-webkit-` declarations by design, and a
    // console full of warnings about them trains the reader to ignore the
    // warnings that matter.
    let severity = if feature.kind() == UnsupportedFeatureKind::VendorProperty {
        DiagnosticSeverity::Info
    } else {
        DiagnosticSeverity::Warning
    };
    Diagnostic::new(
        DiagnosticCode::new(code_for_unsupported(feature.kind())),
        severity,
        DiagnosticSubsystem::Style,
        feature.to_string(),
    )
    .with_recovery(RecoveryAction::Fallback)
}

/// The stable code for a [`CssParseErrorKind`]. One code per kind, never derived
/// from message text, so it survives localization (spec §1.6).
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

/// The stable code for an [`UnsupportedFeatureKind`].
const fn code_for_unsupported(kind: UnsupportedFeatureKind) -> &'static str {
    match kind {
        UnsupportedFeatureKind::AtRule => "css.unsupported_at_rule",
        UnsupportedFeatureKind::Property => "css.unsupported_property",
        UnsupportedFeatureKind::Selector => "css.unsupported_selector",
        UnsupportedFeatureKind::Function => "css.unsupported_function",
        UnsupportedFeatureKind::Unit => "css.unsupported_unit",
        UnsupportedFeatureKind::PseudoElement => "css.unsupported_pseudo_element",
        UnsupportedFeatureKind::VendorProperty => "css.ignored_vendor_property",
    }
}

/// Maps a compiled sheet's selector-associated motion into the runtime's
/// domain-only rule types. The CSS→track lowering already happened in the
/// frontend; this only carries the fields across the crate boundary.
fn collect_motion(
    compiled: &CompiledStylesheet,
    transitions: &mut Vec<CssTransitionRule>,
    keyframes: &mut Vec<CssKeyframeRule>,
) {
    for rule in &compiled.motion_rules {
        let source_order = transitions.len();
        if rule.transition_reset {
            transitions.push(
                CssTransitionRule::reset(rule.selectors.clone())
                    .with_condition(rule.condition.clone())
                    .with_cascade(source_order, rule.transition_important),
            );
        }
        for transition in &rule.transitions {
            transitions.push(
                CssTransitionRule::new(
                    rule.selectors.clone(),
                    transition.property,
                    transition.duration_secs,
                    transition.easing,
                )
                .with_delay(transition.delay_secs)
                .with_condition(rule.condition.clone())
                .with_cascade(source_order, rule.transition_important),
            );
        }
        let mut group = keyframes.len();
        let mut last_reference = None;
        for animation in &rule.animations {
            if last_reference != Some(animation.reference_index) {
                group = keyframes.len();
                last_reference = Some(animation.reference_index);
            }
            let condition = match (&rule.condition, &animation.condition) {
                (Some(rule), Some(definition)) => {
                    Some(obscur_style::matching::RuleCondition::All(vec![
                        rule.clone(),
                        definition.clone(),
                    ]))
                }
                (one, other) => one.as_ref().or(other.as_ref()).cloned(),
            };
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
                )
                .with_condition(condition)
                .with_definition_group(group),
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
