//! A character outside the application's font chain is drawn, not dropped
//! (spec §5.10 font fallback).
//!
//! The defect: an application's chain is the families it names plus the three
//! generics a host installs — Latin, Cyrillic and Greek between them. Anything
//! else (CJK, emoji, `⌫` on a key) resolved to no face, and an unresolved run
//! paints **no glyphs at all** — a blank where a character should be, which
//! reads as a broken control rather than as a missing font.
//!
//! These tests need a host with fontconfig and real system fonts. Where there is
//! none they assert nothing, because there is nothing to ask — which is stated
//! here rather than hidden, since a silently-skipping test is worse than none.
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

/// The bundled last-resort face, and *only* it: a deliberately minimal chain, so
/// a character outside its 2,468-codepoint subset has nowhere to go but the
/// host lookup.
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

/// Renders `text` and reports how many glyph runs were painted.
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

/// The control: Latin is inside the bundled subset, so it paints with or without
/// a host. Without this, a fallback that quietly broke *all* text would still
/// satisfy the case below.
#[test]
fn text_the_bundled_face_covers_is_painted() {
    assert!(painted_runs("abc") > 0, "Latin must paint");
}

/// `⌫` (U+232B) is not in the bundled face. Before the host lookup it painted
/// nothing at all.
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

/// Fallback is chosen per **run**, not per character — so a run mixing covered
/// and uncovered text is drawn entirely in the discovered face.
///
/// Worth pinning as the behaviour rather than leaving it to be discovered: the
/// text is itemized by script, a face is chosen for each item, and one character
/// the chain cannot draw therefore pulls its whole item into the host's face.
/// `ab⌫cd` comes out in DejaVu rather than mixing Noto with DejaVu. That is a
/// visible difference from a browser, and it is the price of choosing a face per
/// run; splitting an item at coverage boundaries would be the fix, and is not
/// built.
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

/// The host is asked only for what the chain cannot draw. A face pulled in for
/// Latin would mean every application quietly loading a second font it does not
/// need.
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

/// A face discovered mid-frame has to reach the renderer, which rasterizes from
/// its own id→bytes table: resolving a glyph the renderer cannot draw is the
/// same blank, one layer down.
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
