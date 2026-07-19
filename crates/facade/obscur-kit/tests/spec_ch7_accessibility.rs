//! **SPECIFICATION-DERIVED** accessibility cases for obscur.kit §7.1 / §7.14.
//!
//! Every expected value below was produced by an agent reading obscur.kit §7.1,
//! §7.4 and §7.14 with **no access to this repository's source**.
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use obscur_kit::accessibility::tree::SemanticSnapshot;
use obscur_kit::geometry::Size;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::view::view::View;
use support::{drive, el, txt, view};

const VW: f32 = 200.0;

const CSS: &str = ".panel { width: 200px; height: 120px } \
                   .button { width: 80px; height: 30px; background-color: #336699 } \
                   .label { width: 120px; height: 20px; color: #000000 }";

fn tree() -> View {
    view(vec![el(
        1,
        "panel",
        vec![
            el(2, "button", vec![txt("Save")]),
            el(3, "label", vec![txt("Name")]),
        ],
    )])
}

/// A renderable, comparable rendering of a semantic snapshot: role, name, state
/// and parentage for every node, in tree order.
fn describe(snapshot: &SemanticSnapshot) -> String {
    let mut rows: Vec<String> = snapshot
        .iter()
        .map(|(id, node)| {
            format!(
                "{}|{:?}|{:?}|{:?}|kids={:?}",
                id.raw(),
                node.role(),
                node.name(),
                node.state(),
                snapshot
                    .children(id)
                    .iter()
                    .map(|c| c.raw())
                    .collect::<Vec<_>>()
            )
        })
        .collect();
    rows.sort();
    rows.join("\n")
}

/// §7.14: "Golden tests should prefer backend-independent snapshots:" /
/// "- accessibility;" with §7.4 "Official headless mode operates without window
/// or GPU."
#[test]
fn the_accessibility_tree_is_identical_with_and_without_a_raster_backend() {
    // (a) Headless with no renderer backend registered at all.
    let no_backend = drive(VW, VW, CSS, &tree());
    let a = describe(&no_backend.semantic_snapshot());

    // (b) Headless through the CPU raster path — the same tree, rasterized.
    let mut cpu = drive(VW, VW, CSS, &tree());
    let mut renderer = CpuRenderer::new(RendererId::from_raw(2));
    let submission = FrameSubmission::new(RenderFrame::new(cpu.snapshot(), Size::new(VW, VW)));
    let target = RenderTarget::bitmap(BitmapTarget::new(Size::new(VW, VW)));
    renderer
        .render(&submission, &target)
        .expect("the CPU backend rasterizes the scene");
    assert!(
        renderer.pixmap().is_some(),
        "case (b) really did rasterize — otherwise it is not a second backend"
    );
    let b = describe(&cpu.semantic_snapshot());

    assert_eq!(
        a, b,
        "the accessibility snapshot is byte-identical with and without rasterization"
    );

    // Nothing about a11y may depend on having painted: capturing it again on the
    // un-rasterized runtime must give the same answer.
    let a_again = describe(&no_backend.semantic_snapshot());
    assert_eq!(
        a, a_again,
        "the accessibility snapshot is stable across repeated captures"
    );

    // And the un-rasterized run must not be empty — a tree computed during paint
    // would be.
    assert!(
        !no_backend.semantic_snapshot().is_empty(),
        "case (a) produces a real accessibility tree even though nothing was rasterized"
    );

    // Keep `cpu` alive through the comparison.
    let _ = cpu.tree().root();
}
