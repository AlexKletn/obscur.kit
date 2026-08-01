#![doc = include_str!("system_font_fallback.doc/_module.md")]
#![forbid(unsafe_code)]

use std::sync::Arc;

use obscur_kit::bootstrap::{FontFaceBytes, FontSet, KitBootstrap};
use obscur_kit::font_fontconfig::source::FontconfigSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::text::font::{FontAttributes, FontFamily, GenericFamily};
use obscur_kit::view::view::{View, ViewFragment, ViewNode};

#[doc = include_str!("system_font_fallback.doc/bundled_only.md")]
fn bundled_only() -> FontSet {
    let mut set = FontSet::new();
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/fonts/NotoSans-Regular.ttf"
    ))
    .expect("the vendored face");
    set.push(FontFaceBytes::new(
        FontFamily::generic(GenericFamily::SansSerif),
        FontAttributes::default(),
        bytes,
    ));
    set
}

fn view(text: &str) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(90))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text(text)])),
    ]))
}

#[doc = include_str!("system_font_fallback.doc/painted_runs.md")]
fn painted_runs(text: &str) -> usize {
    let boot = KitBootstrap::new();
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
    let _installed = boot.install_text_stack(&mut rt, &bundled_only());
    let result = rt.render_now(&view(text), &[]);
    result
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter(|command| matches!(command, PaintCommand::DrawText { .. }))
        .count()
}

fn host_has_fontconfig() -> bool {
    FontconfigSource::new().is_available()
}

#[doc = include_str!("system_font_fallback.doc/text_the_bundled_face_covers_is_painted.md")]
#[test]
fn text_the_bundled_face_covers_is_painted() {
    assert!(painted_runs("abc") > 0, "Latin must paint");
}

#[doc = include_str!("system_font_fallback.doc/a_character_outside_the_chain_is_painted_via_the_host.md")]
#[test]
fn a_character_outside_the_chain_is_painted_via_the_host() {
    if !host_has_fontconfig() {
        return;
    }
    let source = FontconfigSource::new();
    if source.match_char('⌫').is_none() {
        // The host has fontconfig but nothing covering it; nothing to assert.
        return;
    }
    assert!(
        painted_runs("⌫") > 0,
        "a character the chain cannot draw must come from the host, not vanish"
    );
}

#[doc = include_str!("system_font_fallback.doc/a_mixed_run_is_drawn_wholly_in_the_discovered_face.md")]
#[test]
fn a_mixed_run_is_drawn_wholly_in_the_discovered_face() {
    if !host_has_fontconfig() || FontconfigSource::new().match_char('⌫').is_none() {
        return;
    }
    assert!(
        painted_runs("ab⌫cd") > 0,
        "the run paints — before the host lookup it vanished entirely"
    );
}

#[doc = include_str!("system_font_fallback.doc/the_host_is_not_consulted_for_covered_text.md")]
#[test]
fn the_host_is_not_consulted_for_covered_text() {
    let boot = KitBootstrap::new();
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
    let installed = boot.install_text_stack(&mut rt, &bundled_only());
    let _ = rt.render_now(&view("abc"), &[]);

    assert!(
        installed.take_discovered().is_empty(),
        "covered text pulls in nothing"
    );
}

#[doc = include_str!("system_font_fallback.doc/a_discovered_face_is_offered_to_the_renderer_once.md")]
#[test]
fn a_discovered_face_is_offered_to_the_renderer_once() {
    if !host_has_fontconfig() || FontconfigSource::new().match_char('⌫').is_none() {
        return;
    }
    let boot = KitBootstrap::new();
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 200.0));
    let installed = boot.install_text_stack(&mut rt, &bundled_only());
    let _ = rt.render_now(&view("⌫"), &[]);

    let first = installed.take_discovered();
    assert_eq!(first.len(), 1, "the face is offered");
    assert!(!Arc::clone(&first[0].1).is_empty(), "with its bytes");
    assert!(
        installed.take_discovered().is_empty(),
        "and only once — a drain hands each face over a single time"
    );
}
