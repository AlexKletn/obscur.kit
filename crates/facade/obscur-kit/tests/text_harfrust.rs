#![doc = include_str!("text_harfrust.doc/_module.md")]
#![cfg(feature = "text-harfrust")]

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::view::view::{View, ViewFragment, ViewNode};

#[doc = include_str!("text_harfrust.doc/text_view.md")]
fn text_view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(10))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("AB")])),
    ]))
}

#[doc = include_str!("text_harfrust.doc/drawtext_glyph_count.md")]
fn drawtext_glyph_count(runtime: &UiRuntime) -> Option<usize> {
    runtime
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.len()),
            _ => None,
        })
}

#[test]
fn installing_the_harfrust_shaper_through_the_facade_changes_the_run() {
    let view = text_view();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));

    // Default deterministic mock shaper: one glyph per char -> two glyphs for "AB".
    let _ = runtime.render_now(&view, &[]);
    assert_eq!(drawtext_glyph_count(&runtime), Some(2));

    // Build the REAL HarfRust adapter through the facade's feature-gated bootstrap
    // helper and install it via the public runtime seam. The bytes here are not a
    // loadable font, so HarfRust returns an empty run (graceful, infallible) — a
    // count of 0, which differs from the mock's 2. That difference proves the
    // installed adapter is actually driving shaping, through the facade feature.
    let shaper = KitBootstrap::new().text_shaper_from_bytes(vec![0u8; 8]);
    runtime.set_text_shaper(shaper);
    let _ = runtime.render_now(&view, &[]);
    assert_eq!(drawtext_glyph_count(&runtime), Some(0));
}
