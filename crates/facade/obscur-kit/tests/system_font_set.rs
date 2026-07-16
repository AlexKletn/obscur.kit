//! The desktop production path: `KitBootstrap::system_font_set` reads the host's
//! own faces off disk through fontconfig (spec §5.18).
//!
//! # These tests are deliberately weak, and that is the point
//!
//! Everything here depends on what the running machine has installed, so nothing
//! can assert a metric, a glyph id, or a family name without drifting per machine
//! — on the authoring machine `fc-match sans-serif` resolves into
//! `/usr/share/fonts/WindowsFonts/`. So this file asserts only the two properties
//! that must hold on *any* machine: whatever comes back is **loadable**, and
//! whatever comes back **shapes real glyphs**. The numeric assertions live in
//! `text_stack_install.rs` against the vendored faces, which is the split the
//! owner's policy asks for: system fonts for production, vendored fonts for tests.
//!
//! A machine with no fontconfig skips rather than fails — a bare CI container is a
//! legitimate host, not a broken one.
#![cfg(all(
    feature = "font-fontconfig",
    feature = "text-harfrust",
    feature = "text-unicode"
))]
#![forbid(unsafe_code)]

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::font_fontconfig::source::FontconfigSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::text::PositionedGlyph;
use obscur_kit::text::font::GenericFamily;
use obscur_kit::view::view::{View, ViewFragment, ViewNode};

/// Whether this machine can answer font queries at all.
fn fontconfig_present() -> bool {
    FontconfigSource::new().is_available()
}

/// Every face fontconfig hands back must be one the shaping stack can actually
/// parse. This is the `.ttc` guard's end-to-end consequence: a collection would
/// register with no `cmap` coverage and its text would silently vanish, so the
/// enumeration must never emit one.
#[test]
fn every_enumerated_face_is_loadable_by_the_shaping_stack() {
    if !fontconfig_present() {
        eprintln!("skipped: no fontconfig on this machine");
        return;
    }
    let Some(set) = KitBootstrap::new().system_font_set() else {
        eprintln!("skipped: fontconfig nominated no loadable face");
        return;
    };
    assert!(!set.is_empty(), "a Some(set) must not be empty");

    for face in set.faces() {
        // The registry reads a face's `head`/`hhea` at registration and reports a
        // face it could not parse as metric-less, so a real `units_per_em` is
        // exactly "this parsed as a font".
        let mut registry = obscur_kit::text_harfrust::FontRegistry::new();
        let id = registry.register(
            face.family().clone(),
            *face.attributes(),
            std::sync::Arc::clone(face.bytes()),
        );
        let metrics = registry.face_metrics().get(id);
        assert!(
            metrics.is_some_and(|m| m.units_per_em > 0),
            "fontconfig nominated {:?}, which the shaping stack cannot parse — \
             it would resolve nothing and its text would vanish",
            face.family().name()
        );
    }
}

/// The whole desktop path in one line: enumerate the host's faces, install them,
/// and get real glyphs. No family name, glyph id, or metric is asserted — only
/// that *something the host owns* shaped.
#[test]
fn the_host_font_set_installs_and_shapes_real_glyphs() {
    if !fontconfig_present() {
        eprintln!("skipped: no fontconfig on this machine");
        return;
    }
    let boot = KitBootstrap::new();
    let Some(set) = boot.system_font_set() else {
        eprintln!("skipped: fontconfig nominated no loadable face");
        return;
    };

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let installed = boot.install_text_stack(&mut runtime, &set);
    assert_eq!(
        installed.len(),
        set.len(),
        "every enumerated face must get a handle"
    );

    let view = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(10))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("Ab")])),
    ]));
    runtime.render_now(&view, &[]);

    let glyphs: Vec<u32> = runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { run, .. } => {
                Some(run.glyphs().iter().map(PositionedGlyph::glyph_id).collect())
            }
            _ => None,
        })
        .unwrap_or_default();

    assert_eq!(glyphs.len(), 2, "two characters must shape to two glyphs");
    assert!(
        glyphs.iter().all(|&g| g != 0),
        "a `.notdef` (0) means the installed face does not cover ASCII: {glyphs:?}"
    );
    // The mock emits the Unicode scalar; any real face maps 'A' somewhere else.
    // (A face where 'A' genuinely lands on gid 65 would make this vacuous, so it
    // is a support check, not the proof — `text_stack_install.rs` proves the ids.)
    assert!(
        glyphs != vec!['A' as u32, 'b' as u32],
        "the mock shaper is still installed"
    );
}

/// `fc-match` never fails — asked for a family that does not exist it returns the
/// system default with exit status 0. A named-family lookup built on it alone would
/// resolve every absent family to a substituted face, and an author's fallback
/// chain would never be walked.
///
/// The adapter guards this twice over (`fc-list`, which returns nothing for an
/// absent family, plus a name filter), and the guards are redundant: this test only
/// fails once **both** are gone, so it pins the outcome rather than either
/// mechanism. That is what it should pin — the substitution is the bug.
#[test]
fn an_absent_named_family_is_not_silently_substituted() {
    if !fontconfig_present() {
        eprintln!("skipped: no fontconfig on this machine");
        return;
    }
    let source = FontconfigSource::new();
    let absent = source.find_family(
        "ThisFamilyDoesNotExist12345",
        &obscur_kit::text::font::FontAttributes::new(),
    );
    assert!(
        absent.is_none(),
        "an absent family must be None, not a substitute — got {absent:?}"
    );

    // The contrast that gives the assertion its teeth: the *generic* path is
    // allowed (and expected) to substitute, and does answer.
    assert!(
        source.match_generic(GenericFamily::SansSerif).is_some(),
        "a generic must still resolve on a machine that has fontconfig"
    );
}
