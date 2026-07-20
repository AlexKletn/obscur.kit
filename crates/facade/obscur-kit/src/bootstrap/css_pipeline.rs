//! [`CssPipeline`]: the host-side CSS path — parse (cached) → compile → install
//! onto a [`UiRuntime`](crate::runtime::UiRuntime).
//!
//! The CSS frontend and the runtime are deliberately unaware of each other: the
//! frontend emits `obscur-css` types, the runtime accepts only domain types
//! (`obscur-style`'s [`Stylesheet`] and `obscur-runtime`'s [`CssMotionRules`]).
//! Joining them is a fixed piece of plumbing that every CSS-rendering host needs
//! and none should hand-write — so it lives here, next to
//! [`install_vector_resources`](super::KitBootstrap::install_vector_resources),
//! rather than in each application.
//!
//! Two things are easy to get wrong by hand and are why this type exists:
//!
//! - **Motion is dropped.** A host that installs only `compiled.matchable` gets
//!   a runtime that never sees the sheet's `transition`/`@keyframes`, so CSS
//!   authored motion silently does nothing (§4.17).
//! - **Sources are re-parsed.** A reload recompiles the whole sheet set, but only
//!   the edited source's *text* changed; re-parsing the rest is exactly the work
//!   §4.12's [`StylesheetCache`] exists to skip.

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

/// The host's CSS path: owns the §4.12 parse cache across reloads, compiles each
/// source, and installs both halves of the result — the matchable rules **and**
/// the CSS-authored motion — on a runtime.
///
/// # Lifetime
///
/// One pipeline per runtime, kept alive for the app's lifetime: the cache it owns
/// is what makes a reload cheap, and dropping it between reloads throws that away.
///
/// # When to install
///
/// On a (re)compile — startup, and each time a source's text changes — not per
/// frame. [`install`](Self::install) replaces the runtime's motion rules, which
/// resets the per-node change-detection baselines transitions observe, so calling
/// it every frame would leave every transition permanently at its first frame.
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

    /// Compiles `sources` in order and installs the result on `runtime`: the
    /// matchable rules via
    /// [`set_stylesheets`](crate::runtime::UiRuntime::set_stylesheets), and the
    /// sheet's `transition`/`@keyframes` via
    /// [`set_css_motion`](crate::runtime::UiRuntime::set_css_motion) so the
    /// runtime actually drives them (§4.17).
    ///
    /// Each source is parsed only if the cache has no entry at its
    /// [`version`](CssSource::version); an
    /// [`ANONYMOUS`](obscur_css::source::SourceId::ANONYMOUS) source has no stable
    /// identity and is therefore always re-parsed. Everything downstream of the
    /// parse (compile, lower, install) runs every call — this caches parsing, which
    /// is what §4.12 specifies.
    ///
    /// A malformed sheet compiles to whatever it recovered — the rejected parts
    /// never reach the runtime, and the surviving rules do (spec §1.6 "failures
    /// should be localized to the smallest safe scope"). Each rejected part is
    /// **also** raised as a `Style`-subsystem [`Diagnostic`] on the runtime's
    /// §1.6 channel via
    /// [`report_diagnostic`](crate::runtime::UiRuntime::report_diagnostic): a
    /// syntactic error (`color: ;`) and a recognised-but-unsupported construct
    /// (`width: 10qq`, `float: left`) each report one diagnostic. The runtime
    /// receives only compiled rules and never a sheet's source, so this — the one
    /// place holding both the errors and the runtime — is where they reach the
    /// channel. The same lists remain readable on [`errors`](Self::errors) and
    /// [`unsupported`](Self::unsupported).
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

    /// Drops every cached parse, forcing the next [`install`](Self::install) to
    /// re-parse from source text.
    ///
    /// The escape hatch for an owner that edited a sheet without stamping a new
    /// [`version`](CssSource::version) — §4.12's explicit invalidation.
    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
    }
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
    Diagnostic::new(
        DiagnosticCode::new(code_for_unsupported(feature.kind())),
        DiagnosticSeverity::Warning,
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
