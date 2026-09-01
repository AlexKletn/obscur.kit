#![doc = include_str!("system_theme_absence.doc/_module.md")]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::capability::SupportLevel;
use obscur_kit::diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticSink};
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::system::appearance::{
    AppearanceProvider, ColorScheme, SystemAppearance, SystemFont,
};

#[doc = include_str!("system_theme_absence.doc/Collected.md")]
type Collected = Arc<Mutex<Vec<(String, DiagnosticSeverity)>>>;

#[doc = include_str!("system_theme_absence.doc/Collecting.md")]
struct Collecting(Collected);

impl DiagnosticSink for Collecting {
    fn emit(&self, diagnostic: &Diagnostic) {
        if let Ok(mut seen) = self.0.lock() {
            seen.push((diagnostic.code.as_str().to_owned(), diagnostic.severity));
        }
    }
}

#[doc = include_str!("system_theme_absence.doc/runtime_watching.md")]
fn runtime_watching() -> (UiRuntime, Collected) {
    let seen: Collected = Arc::new(Mutex::new(Vec::new()));
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 300.0));
    runtime.set_diagnostic_sink(Box::new(Collecting(Arc::clone(&seen))));
    (runtime, seen)
}

#[doc = include_str!("system_theme_absence.doc/codes.md")]
fn codes(seen: &Collected) -> Vec<(String, DiagnosticSeverity)> {
    seen.lock().expect("no test thread panicked").clone()
}

#[doc = include_str!("system_theme_absence.doc/Barren.md")]
struct Barren;

impl AppearanceProvider for Barren {
    fn appearance(&self) -> SystemAppearance {
        SystemAppearance::default()
    }
    fn support(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

#[doc = include_str!("system_theme_absence.doc/Colourless.md")]
struct Colourless;

impl AppearanceProvider for Colourless {
    fn appearance(&self) -> SystemAppearance {
        SystemAppearance {
            color_scheme: Some(ColorScheme::Dark),
            font: Some(SystemFont::new("Cantarell", Some(11.0))),
            ..SystemAppearance::default()
        }
    }
    fn support(&self) -> SupportLevel {
        SupportLevel::Limited
    }
}

#[test]
fn a_machine_with_no_appearance_source_is_not_an_error() {
    // The claim in full: no panic, no `Result`, and an application that is
    // simply unstyled by the desktop rather than broken by it.
    let (mut runtime, _seen) = runtime_watching();
    let vars = KitBootstrap::new().install_system_theme_from(&mut runtime, &Barren);

    assert!(
        vars.is_empty(),
        "no desktop said anything, so nothing is set"
    );
    assert_eq!(vars.stylesheet(), "");
    assert!(
        runtime.has_default_stylesheet(),
        "and the UA sheet is still installed — the application keeps its own styling"
    );
}

#[test]
fn it_says_so_once_at_warning_level_and_no_louder() {
    // "At most a warning" as an assertion rather than an intention: nothing
    // above `Warning` may be emitted for an absence.
    let (mut runtime, seen) = runtime_watching();
    KitBootstrap::new().install_system_theme_from(&mut runtime, &Barren);

    let reported = codes(&seen);
    assert!(
        reported
            .iter()
            .any(|(code, _)| code == "platform.appearance.no_source"),
        "the absence is reported: {reported:?}"
    );
    for (code, severity) in &reported {
        assert!(
            matches!(
                severity,
                DiagnosticSeverity::Trace | DiagnosticSeverity::Info | DiagnosticSeverity::Warning
            ),
            "{code} came in at {severity:?}, and an absent desktop is not an error"
        );
    }
}

#[test]
fn a_desktop_that_states_no_colours_is_a_note_rather_than_a_complaint() {
    // The ordinary GNOME outcome. It must not read as something wrong, so it
    // sits below the default stderr floor.
    let (mut runtime, seen) = runtime_watching();
    let vars = KitBootstrap::new().install_system_theme_from(&mut runtime, &Colourless);

    assert!(!vars.is_empty(), "the scheme and the font did arrive");
    assert_eq!(vars.get("--obscur-sys-color-scheme"), Some("dark"));
    assert_eq!(vars.get("--obscur-sys-window-bg"), None);

    let reported = codes(&seen);
    let palette = reported
        .iter()
        .find(|(code, _)| code == "platform.appearance.no_palette")
        .expect("the missing palette is noted");
    assert_eq!(palette.1, DiagnosticSeverity::Info);
    assert!(
        !reported
            .iter()
            .any(|(code, _)| code == "platform.appearance.no_source"),
        "a source did answer, so it must not be reported as absent: {reported:?}"
    );
}

#[test]
fn the_two_absences_are_told_apart() {
    // "Nothing to ask" and "asked, and it said nothing" are different facts and
    // a host may want to act differently on them, so they carry different codes.
    let (mut absent, absent_seen) = runtime_watching();
    KitBootstrap::new().install_system_theme_from(&mut absent, &Barren);

    let (mut hand_made, made_seen) = runtime_watching();
    KitBootstrap::new().install_system_theme(&mut hand_made, &SystemAppearance::default());

    let absent_codes: Vec<String> = codes(&absent_seen).into_iter().map(|(c, _)| c).collect();
    let made_codes: Vec<String> = codes(&made_seen).into_iter().map(|(c, _)| c).collect();

    assert!(
        absent_codes
            .iter()
            .any(|c| c == "platform.appearance.no_source")
    );
    assert!(made_codes.iter().any(|c| c == "platform.appearance.empty"));
    assert!(
        !made_codes
            .iter()
            .any(|c| c == "platform.appearance.no_source"),
        "an appearance handed over directly says nothing about a source: {made_codes:?}"
    );
}
