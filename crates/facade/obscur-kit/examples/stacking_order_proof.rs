//! **Proof: CSS 2.1 §9.9 seven-layer paint order (catalog bug B2).**
//!
//! Two independent demonstrations sit side by side, driven through the REAL scene
//! coordinator ([`SceneCoordinator::compose`]) — the same code path the runtime
//! uses — then rasterized headless through the software [`CpuRenderer`] (no GPU) to
//! `target/stacking_order_proof.png`.
//!
//! **Left — the lift (§9.9 layer 6 above layers 1–5).** A positioned `z:auto` box
//! `B` (BLUE) is the FIRST child; a non-positioned in-flow block `A` (GREEN) is the
//! SECOND child and overlaps it. Per §9.9 a positioned `z:auto`/`0` box paints
//! ABOVE non-positioned siblings regardless of tree order, so `B` must show ON TOP
//! of `A` in the overlap. The old flat `(z, tree_index)` sort painted `A` (later in
//! the tree) on top — the bug.
//!
//! **Right — negative z below non-positioned (§9.9 layer 2).** A non-positioned
//! block `S` (YELLOW) is a child; a positioned box `N` (RED) with `z-index: -2`
//! comes LATER in the tree and overlaps it. A negative-z positioned box paints
//! UNDER non-positioned content, so `S` must show ON TOP of `N` even though `N`
//! comes later in document order.
//!
//! Before writing the PNG it machine-checks the pixels at each overlap.
//!
//! Run it:
//! `cargo run -p obscur-kit --example stacking_order_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId};
use obscur_kit::layout::model::LayoutBox;
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::orchestration::{ComputedStyles, LayoutBoxes, SceneCoordinator};
use obscur_kit::scene::command::DisplayList;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};
use obscur_kit::style::computed::{ComputedStyle, ComputedValue};
use obscur_kit::style::property::{PropertyId, StyleColor, layout_keyword};
use obscur_kit::tree::node::{LogicalParent, NodeKind};
use obscur_kit::tree::tree::RetainedTree;

/// The output image extent.
const VIEWPORT: Size = Size::new(400.0, 200.0);

fn main() {
    println!("== obscur-kit §9.9 stacking-order proof (headless CPU) ==\n");

    let blue = StyleColor::rgba(0.20, 0.40, 0.90, 1.0); // B: positioned z:auto (lifts)
    let green = StyleColor::rgba(0.20, 0.75, 0.35, 1.0); // A: non-positioned block
    let yellow = StyleColor::rgba(0.95, 0.85, 0.15, 1.0); // S: non-positioned block
    let red = StyleColor::rgba(0.90, 0.20, 0.20, 1.0); // N: positioned z:-2 (demoted)

    // Left demonstration: B (positioned z:auto) is the FIRST child, A (static
    // block) the SECOND and overlapping. Right demonstration: S (static) is a
    // child, N (positioned z:-2) comes LATER and overlaps it.
    let box_b = Rect::new(Point::new(30.0, 30.0), Size::new(100.0, 100.0));
    let box_a = Rect::new(Point::new(80.0, 80.0), Size::new(100.0, 100.0));
    let box_s = Rect::new(Point::new(230.0, 30.0), Size::new(100.0, 100.0));
    let box_n = Rect::new(Point::new(280.0, 80.0), Size::new(100.0, 100.0));

    let mut tree = RetainedTree::new();
    let root = tree.alloc(NodeKind::Root);
    tree.set_root(root);
    // Child order matters: B before A (left), S before N (right).
    let b = element(&mut tree, root, 1);
    let a = element(&mut tree, root, 2);
    let s = element(&mut tree, root, 3);
    let n = element(&mut tree, root, 4);

    let mut computed: ComputedStyles = HashMap::new();
    // The root paints NO background: in this z-primary flat model a `z:-2` box
    // sorts below EVERYTHING at z=0 (including a full-viewport root backdrop), so an
    // opaque root background would hide the negative-z box entirely. This mirrors
    // the documented §9.9 limitation — layer 1 (root bg) vs layer 2 (negative z)
    // cannot be expressed when `z` is the primary sort key. The bitmap's clear
    // colour shows through instead.
    computed.insert(root, ComputedStyle::new());
    computed.insert(b, positioned_auto(blue)); // positioned, z:auto → §9.9 layer 6
    computed.insert(a, background(green)); // non-positioned block → layer 3
    computed.insert(s, background(yellow)); // non-positioned block → layer 3
    computed.insert(n, positioned_z(red, -2.0)); // positioned z:-2 → layer 2 (below)

    let mut boxes: LayoutBoxes = HashMap::new();
    boxes.insert(
        root,
        LayoutBox::new(Rect::new(Point::new(0.0, 0.0), VIEWPORT)),
    );
    boxes.insert(b, LayoutBox::new(box_b));
    boxes.insert(a, LayoutBox::new(box_a));
    boxes.insert(s, LayoutBox::new(box_s));
    boxes.insert(n, LayoutBox::new(box_n));

    // Drive the REAL coordinator: it computes the §9.9 layer rank and sorts by
    // `(z, layer, tree_index, order)`.
    let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
    println!("[scene] {} paint command(s)", list.commands().len());

    let pixmap = render(list);

    // Overlap sample points.
    let left_overlap = (100u32, 100u32); // inside BOTH B and A → must be BLUE (B on top)
    let b_only = (45u32, 45u32); // inside B only → BLUE
    let a_only = (160u32, 160u32); // inside A only → GREEN
    let right_overlap = (300u32, 100u32); // inside BOTH S and N → must be YELLOW (S on top)
    let s_only = (245u32, 45u32); // inside S only → YELLOW
    let n_only = (350u32, 150u32); // inside N only → RED

    verify(
        &pixmap,
        left_overlap,
        b_only,
        a_only,
        right_overlap,
        s_only,
        n_only,
    );

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("stacking_order_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — positioned z:auto ON TOP of a later non-positioned block; negative-z UNDER an earlier one",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

/// Allocates an element node under `parent`, wiring the child + logical-parent
/// links, and returns its id.
fn element(tree: &mut RetainedTree, parent: NodeId, raw: u64) -> NodeId {
    let id = tree.alloc(NodeKind::Element {
        component: ComponentId::from_raw(raw),
    });
    if let Some(node) = tree.get_mut(parent) {
        node.push_child(id);
    }
    if let Some(node) = tree.get_mut(id) {
        node.set_logical_parent(LogicalParent::of(parent));
    }
    id
}

/// A computed style carrying just a `background-color` (a non-positioned block).
fn background(color: StyleColor) -> ComputedStyle {
    let mut style = ComputedStyle::new();
    style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
    style
}

/// A positioned (`position: absolute`) box with `z-index: auto` — §9.9 layer 6.
fn positioned_auto(color: StyleColor) -> ComputedStyle {
    let mut style = background(color);
    style.set(
        PropertyId::Position,
        ComputedValue::keyword(layout_keyword::POSITION_ABSOLUTE),
    );
    style
}

/// A positioned box with an explicit integer `z-index`.
fn positioned_z(color: StyleColor, z: f32) -> ComputedStyle {
    let mut style = positioned_auto(color);
    style.set(PropertyId::ZIndex, ComputedValue::number(z));
    style
}

/// Renders `list` through the software renderer into a pixmap at [`VIEWPORT`].
fn render(list: DisplayList) -> tiny_skia::Pixmap {
    let snapshot = SceneSnapshot::new(list, SceneGeneration::from_raw(1));
    let frame = RenderFrame::new(snapshot, VIEWPORT);
    let submission = FrameSubmission::new(frame);
    let target = RenderTarget::bitmap(BitmapTarget::new(VIEWPORT));
    let mut renderer = CpuRenderer::new(RendererId::from_raw(1));
    renderer
        .render(&submission, &target)
        .expect("cpu render succeeds");
    renderer.pixmap().expect("a rasterized frame").clone()
}

/// Reads a pixel as `(r, g, b)` bytes.
fn rgb(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
    let p = pixmap.pixel(x, y).expect("pixel in range");
    (p.red(), p.green(), p.blue())
}

fn is_blue(p: (u8, u8, u8)) -> bool {
    p.2 > 140 && p.0 < 130 && p.1 < 160
}
fn is_green(p: (u8, u8, u8)) -> bool {
    p.1 > 140 && p.0 < 130 && p.2 < 150
}
fn is_yellow(p: (u8, u8, u8)) -> bool {
    p.0 > 180 && p.1 > 160 && p.2 < 120
}
fn is_red(p: (u8, u8, u8)) -> bool {
    p.0 > 180 && p.1 < 120 && p.2 < 120
}

/// Machine-checks the §9.9 paint order at each sample point.
#[allow(clippy::too_many_arguments)]
fn verify(
    pixmap: &tiny_skia::Pixmap,
    left_overlap: (u32, u32),
    b_only: (u32, u32),
    a_only: (u32, u32),
    right_overlap: (u32, u32),
    s_only: (u32, u32),
    n_only: (u32, u32),
) {
    // Left: the positioned z:auto box B (blue) paints ON TOP of the later
    // non-positioned block A (green) at the overlap — the fix for B2.
    let lo = rgb(pixmap, left_overlap.0, left_overlap.1);
    assert!(
        is_blue(lo),
        "B2 NOT fixed: overlap {left_overlap:?} should be BLUE (positioned z:auto on top), got {lo:?}"
    );
    assert!(
        is_blue(rgb(pixmap, b_only.0, b_only.1)),
        "B-only should be blue"
    );
    assert!(
        is_green(rgb(pixmap, a_only.0, a_only.1)),
        "A-only should be green"
    );

    // Right: the negative-z positioned box N (red) paints UNDER the earlier
    // non-positioned block S (yellow) at the overlap, despite coming LATER.
    let ro = rgb(pixmap, right_overlap.0, right_overlap.1);
    assert!(
        is_yellow(ro),
        "negative-z NOT below: overlap {right_overlap:?} should be YELLOW (non-positioned on top), got {ro:?}"
    );
    assert!(
        is_yellow(rgb(pixmap, s_only.0, s_only.1)),
        "S-only should be yellow"
    );
    assert!(
        is_red(rgb(pixmap, n_only.0, n_only.1)),
        "N-only should be red"
    );

    println!(
        "[verify] left overlap {left_overlap:?} is BLUE — positioned z:auto lifts above a later non-positioned block"
    );
    println!(
        "[verify] right overlap {right_overlap:?} is YELLOW — negative-z positioned paints under an earlier non-positioned block"
    );
    println!("[verify] OK — CSS 2.1 §9.9 seven-layer paint order");
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
