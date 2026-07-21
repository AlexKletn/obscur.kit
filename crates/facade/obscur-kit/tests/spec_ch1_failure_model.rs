//! **SPECIFICATION-DERIVED** failure-model cases for obscur.kit §1.6.
//!
//! Read this before touching anything in the file.
//!
//! Every expected value below was produced by an agent reading obscur.kit §1
//! (Architecture & boundaries) with **no access to this repository's source**.
//! The cases are a statement of what the implementation owes the spec, not a
//! record of what it currently does. A failure here is a defect report about the
//! toolkit (or, occasionally, about the spec), and must not be "corrected" by
//! editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

#[path = "spec_ch1_support/mod.rs"]
mod support;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::css::source::CssSource;
use obscur_kit::diagnostics::{DiagnosticSubsystem, RecoveryAction};
use obscur_kit::localization::locale::LanguageTag;
use obscur_kit::primitives::Image;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewProperty, ViewValue};

use support::{
    Collected, approx, at, compile, el, geom, install_missing_resources, runtime_with_id, txt, view,
};

/// The viewport every case in this file renders under.
const VP: f32 = 1000.0;

/// A view holding one `Image` primitive classed `img`, sourced at `src`.
fn image_view(src: &str) -> View {
    let node = Image::new(src)
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

/// §1.6: "panic is not normal control flow" and "failures should be localized to
/// the smallest safe scope".
#[test]
fn arch_16_a_malformed_declaration_kills_neither_its_rule_nor_the_sheet() {
    let css = ".a { width: 10px } \
               .b { color: ; width: 10qq; height: 20px } \
               .c { width: 30px }";

    let mut rt = runtime_with_id(1, VP, VP);
    let diagnostics = Collected::install(&mut rt);

    let mut pipeline = CssPipeline::new();
    pipeline.install(&mut rt, &[CssSource::anonymous(css)]);

    let tree = view(vec![
        el(10, "a", vec![]),
        el(11, "b", vec![]),
        el(12, "c", vec![]),
    ]);
    rt.render_now(&tree, pipeline.sheets());

    // The valid rules either side of the malformed one survive.
    approx(geom(&rt, at(&rt, &[0])).2, 10.0, ".a keeps width 10px");
    approx(geom(&rt, at(&rt, &[2])).2, 30.0, ".c keeps width 30px");
    // And the one valid declaration inside the malformed rule survives.
    approx(geom(&rt, at(&rt, &[1])).3, 20.0, ".b keeps height 20px");

    // Exactly one diagnostic per invalid declaration, attributed to style.
    let style = diagnostics.from(DiagnosticSubsystem::Style);
    assert_eq!(
        style.len(),
        2,
        "one style diagnostic per invalid declaration (`color: ;` and \
         `width: 10qq`), got {style:?}"
    );
}

/// §1.6: "diagnostics include severity, subsystem, stable code,
/// node/resource/window context and recovery action."
#[test]
fn arch_17_a_missing_resource_diagnostic_carries_all_five_required_fields() {
    let mut rt = runtime_with_id(1, VP, VP);
    install_missing_resources(&mut rt);
    let diagnostics = Collected::install(&mut rt);

    let sheet = compile(".img { width: 50px; height: 50px }");
    rt.render_now(&image_view("missing-logo"), std::slice::from_ref(&sheet));
    // With no blocking executor the missing-image load defers out of the frame
    // phase (§3.4/§5.15); draining it on poll is what fails the resource and emits
    // its one diagnostic. (The sanctioned frame-purity poll, matching arch_20.)
    let _ = rt.poll_resource_tasks();

    let all = diagnostics.all();
    assert_eq!(
        all.len(),
        1,
        "a missing resource reports exactly one diagnostic, got {all:?}"
    );
    let d = &all[0];

    // 1. severity — a value from the severity enum (any variant is fine; the
    //    field must exist and be typed, which the type system already gives).
    // 2. subsystem — the resources subsystem.
    assert_eq!(
        d.subsystem,
        DiagnosticSubsystem::Resources,
        "attributed to the resources subsystem"
    );
    // 3. stable code — machine-readable, and not the human message.
    assert!(!d.code.as_str().is_empty(), "a stable code is present");
    assert_ne!(
        d.code.as_str(),
        d.message.as_str(),
        "the code is not the human message"
    );
    // 4. context — resource AND node AND window.
    assert!(
        d.context.resource.is_some(),
        "context names the missing resource"
    );
    assert!(
        d.context.node.is_some(),
        "context names the node that referenced it"
    );
    assert!(
        d.context.window.is_some(),
        "context names the window it happened in"
    );
    // 5. recovery action — non-empty, i.e. something other than "no recovery".
    assert_ne!(
        d.recovery,
        RecoveryAction::None,
        "a recovery action describing what was done"
    );
}

/// §1.6: "diagnostics include ... stable code ..." — the code must not be
/// derived from the (localized) human message, nor from the call site.
#[test]
fn arch_18_the_stable_code_survives_locale_and_call_site() {
    let sheet = compile(".img { width: 50px; height: 50px }");

    // (a) en-US, from an image node's source.
    let mut en = runtime_with_id(1, VP, VP);
    install_missing_resources(&mut en);
    let en_diagnostics = Collected::install(&mut en);
    en.set_locale(&LanguageTag::parse("en-US").expect("a valid tag"));
    en.render_now(&image_view("missing-logo"), std::slice::from_ref(&sheet));
    // The sanctioned frame-purity poll: the no-executor load defers, and draining
    // it on poll is what fails the resource and emits its diagnostic (§3.4/§5.15).
    let _ = en.poll_resource_tasks();

    // (b) ru-RU, same failure.
    let mut ru = runtime_with_id(2, VP, VP);
    install_missing_resources(&mut ru);
    let ru_diagnostics = Collected::install(&mut ru);
    ru.set_locale(&LanguageTag::parse("ru-RU").expect("a valid tag"));
    ru.render_now(&image_view("missing-logo"), std::slice::from_ref(&sheet));
    let _ = ru.poll_resource_tasks();

    // (c) a different call site: a CSS `background-image` reference.
    let bg_sheet =
        compile(".bg { width: 50px; height: 50px; background-image: url(missing-logo) }");
    let mut bg = runtime_with_id(3, VP, VP);
    install_missing_resources(&mut bg);
    let bg_diagnostics = Collected::install(&mut bg);
    bg.render_now(
        &view(vec![el(10, "bg", vec![])]),
        std::slice::from_ref(&bg_sheet),
    );
    let _ = bg.poll_resource_tasks();

    let a = en_diagnostics.all();
    let b = ru_diagnostics.all();
    let c = bg_diagnostics.all();
    assert_eq!(a.len(), 1, "en-US reports one diagnostic, got {a:?}");
    assert_eq!(b.len(), 1, "ru-RU reports one diagnostic, got {b:?}");
    assert_eq!(
        c.len(),
        1,
        "the background-image call site reports one diagnostic, got {c:?}"
    );

    assert_eq!(
        a[0].code.as_str(),
        b[0].code.as_str(),
        "the stable code is identical across locales"
    );
    assert_eq!(
        a[0].code.as_str(),
        c[0].code.as_str(),
        "the stable code is identical across call sites"
    );
    assert_ne!(
        a[0].message, b[0].message,
        "the human-readable message differs between en-US and ru-RU"
    );
}

/// §1.6: "fallback and recovery are preferred where correctness can be
/// maintained."
#[test]
fn arch_20_a_missing_image_falls_back_without_disturbing_the_column() {
    let mut rt = runtime_with_id(1, VP, VP);
    install_missing_resources(&mut rt);
    let diagnostics = Collected::install(&mut rt);

    let sheet = compile(
        ".col { display: block; width: 100px } \
         .cell { display: block; width: 100px; height: 100px }",
    );

    // Three declared 100x100 nodes; the middle one is an image with no resource.
    let missing = Image::new("missing-logo")
        .into_view_node()
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("cell".into())));
    let tree = view(vec![el(
        10,
        "col",
        vec![
            el(11, "cell", vec![txt("first")]),
            missing,
            el(13, "cell", vec![txt("third")]),
        ],
    )]);

    let result = rt.render_now(&tree, std::slice::from_ref(&sheet));
    assert!(
        result.is_balanced(),
        "the frame completes rather than erroring out"
    );
    // With no blocking executor the missing-image load is deferred out of the
    // frame phase (§3.4/§5.15); draining it on poll is what fails the resource and
    // emits its one recovery-bearing diagnostic. Layout is unaffected either way.
    let _ = rt.poll_resource_tasks();

    // Every node keeps its declared geometry; node 3's top edge stays at y=200.
    let (_, y1, w1, h1) = geom(&rt, at(&rt, &[0, 0]));
    let (_, y2, w2, h2) = geom(&rt, at(&rt, &[0, 1]));
    let (_, y3, w3, h3) = geom(&rt, at(&rt, &[0, 2]));
    approx(y1, 0.0, "node 1 top");
    approx(y2, 100.0, "node 2 top");
    approx(y3, 200.0, "node 3 top — unchanged by the missing image");
    for (i, (w, h)) in [(w1, h1), (w2, h2), (w3, h3)].into_iter().enumerate() {
        approx(w, 100.0, &format!("node {} width", i + 1));
        approx(h, 100.0, &format!("node {} height", i + 1));
    }

    // Exactly one resources diagnostic, carrying a recovery action.
    let resources = diagnostics.from(DiagnosticSubsystem::Resources);
    assert_eq!(
        resources.len(),
        1,
        "one resources diagnostic for the one missing image, got {resources:?}"
    );
    assert_ne!(
        resources[0].recovery,
        RecoveryAction::None,
        "the diagnostic names the recovery that kept the frame alive"
    );
}
