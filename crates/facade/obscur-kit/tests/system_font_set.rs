#![doc = include_str!("system_font_set.doc/_module.md")]
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

#[doc = include_str!("system_font_set.doc/fontconfig_present.md")]
fn fontconfig_present() -> bool {
    FontconfigSource::new().is_available()
}

#[doc = include_str!("system_font_set.doc/every_enumerated_face_is_loadable_by_the_shaping_stack.md")]
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

#[doc = include_str!("system_font_set.doc/the_host_font_set_installs_and_shapes_real_glyphs.md")]
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

#[doc = include_str!("system_font_set.doc/an_absent_named_family_is_not_silently_substituted.md")]
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
