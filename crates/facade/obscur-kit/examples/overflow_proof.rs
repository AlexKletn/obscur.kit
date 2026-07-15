//! **Proof: CSS `overflow` clips a box's subtree.**
//!
//! Two 100×60 boxes sit side by side, each holding a child that extends BEYOND
//! its box (down and to the right). The left box is `overflow: hidden`; the right
//! box is `overflow: visible`. This drives the REAL scene coordinator
//! ([`SceneCoordinator::compose`]) — the same code path the runtime uses — which
//! brackets a non-`visible` node's subtree with a `PushClip`/`PopClip` around its
//! border box. It then rasterizes the composed display list headless through the
//! software [`CpuRenderer`] (no GPU) and writes `target/overflow_proof.png`.
//!
//! Before writing it machine-checks the pixels: (1) in the left box's overflow
//! region (where the child spills past the box) the pixel is the WHITE background
//! — the child was CLIPPED; (2) in the right box's mirror overflow region the
//! pixel is GREEN — the child OVERFLOWS; and (3) inside the left box the child's
//! green still shows within the clip.
//!
//! Run it:
//! `cargo run -p obscur-kit --example overflow_proof --features render-cpu`
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::PathBuf;

use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::ComponentId;
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
const VIEWPORT: Size = Size::new(380.0, 160.0);

fn main() {
    println!("== obscur-kit overflow clipping proof (headless CPU) ==\n");

    let white = StyleColor::rgba(1.0, 1.0, 1.0, 1.0);
    let blue = StyleColor::rgba(0.20, 0.40, 0.85, 1.0); // the box fill
    let green = StyleColor::rgba(0.20, 0.75, 0.35, 1.0); // the (over)flowing child

    // A container root (white, no clip) holds the two 100×60 boxes. Each box has a
    // child offset down-and-right so it spills past the box's bottom-right edge.
    let box_hidden = Rect::new(Point::new(30.0, 40.0), Size::new(100.0, 60.0));
    let child_hidden = Rect::new(Point::new(90.0, 80.0), Size::new(85.0, 70.0));
    let box_visible = Rect::new(Point::new(230.0, 40.0), Size::new(100.0, 60.0));
    let child_visible = Rect::new(Point::new(290.0, 80.0), Size::new(85.0, 70.0));

    let mut tree = RetainedTree::new();
    let root = tree.alloc(NodeKind::Root);
    tree.set_root(root);
    let hidden = element(&mut tree, root, 1);
    let hidden_child = element(&mut tree, hidden, 2);
    let visible = element(&mut tree, root, 3);
    let visible_child = element(&mut tree, visible, 4);

    let mut computed: ComputedStyles = HashMap::new();
    computed.insert(root, background(white));
    computed.insert(
        hidden,
        background_overflow(blue, layout_keyword::OVERFLOW_HIDDEN),
    );
    computed.insert(hidden_child, background(green));
    computed.insert(
        visible,
        background_overflow(blue, layout_keyword::OVERFLOW_VISIBLE),
    );
    computed.insert(visible_child, background(green));

    let mut boxes: LayoutBoxes = HashMap::new();
    boxes.insert(
        root,
        LayoutBox::new(Rect::new(Point::new(0.0, 0.0), VIEWPORT)),
    );
    boxes.insert(hidden, LayoutBox::new(box_hidden));
    boxes.insert(hidden_child, LayoutBox::new(child_hidden));
    boxes.insert(visible, LayoutBox::new(box_visible));
    boxes.insert(visible_child, LayoutBox::new(child_visible));

    // Drive the REAL coordinator: it emits the PushClip/PopClip for the hidden box.
    let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
    let clips = list
        .commands()
        .iter()
        .filter(|c| matches!(c, obscur_kit::scene::command::PaintCommand::PushClip(_)))
        .count();
    println!(
        "[scene] {} paint command(s) · {clips} PushClip (exactly one — the hidden box)",
        list.commands().len()
    );

    let pixmap = render(list);

    // Overflow region of each box: where the child spills past the box's right
    // edge, at the child's vertical band. Left of the child's left edge (90) and
    // right of the box's right edge.
    let hidden_spill = (150u32, 120u32); // x within child (90..175) but > box right (130)
    let visible_spill = (350u32, 120u32); // mirror: x within child (290..375) but > box right (330)
    let hidden_overlap = (100u32, 90u32); // inside BOTH the hidden box and its child

    verify_clipped(&pixmap, hidden_spill, visible_spill, hidden_overlap);

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("overflow_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "\n[png] wrote {} × {} → {} ({} bytes) — left child CLIPPED, right child OVERFLOWS",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

/// Allocates an element node under `parent`, wiring the child + logical-parent
/// links, and returns its id (mirrors the coordinator's own test helper).
fn element(
    tree: &mut RetainedTree,
    parent: obscur_kit::identity::NodeId,
    raw: u64,
) -> obscur_kit::identity::NodeId {
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

/// A computed style carrying just a `background-color`.
fn background(color: StyleColor) -> ComputedStyle {
    let mut style = ComputedStyle::new();
    style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
    style
}

/// A computed style carrying a `background-color` and an `overflow` keyword.
fn background_overflow(
    color: StyleColor,
    overflow: obscur_kit::identity::InternedSymbol,
) -> ComputedStyle {
    let mut style = background(color);
    style.set(PropertyId::Overflow, ComputedValue::keyword(overflow));
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

/// Whether a pixel reads as the (over)flowing child's green.
fn is_green(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> bool {
    pixmap
        .pixel(x, y)
        .is_some_and(|p| p.green() > 140 && p.red() < 130 && p.blue() < 150)
}

/// Whether a pixel reads as the white container background.
fn is_white(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> bool {
    pixmap
        .pixel(x, y)
        .is_some_and(|p| p.red() > 200 && p.green() > 200 && p.blue() > 200)
}

/// Asserts the hidden box clipped its child while the visible box did not.
fn verify_clipped(
    pixmap: &tiny_skia::Pixmap,
    hidden_spill: (u32, u32),
    visible_spill: (u32, u32),
    hidden_overlap: (u32, u32),
) {
    // (1) The hidden box's overflow region is the WHITE background — the child was
    // clipped away and does not spill past the box.
    assert!(
        is_white(pixmap, hidden_spill.0, hidden_spill.1)
            && !is_green(pixmap, hidden_spill.0, hidden_spill.1),
        "overflow:hidden did NOT clip — expected white at {hidden_spill:?}, found child ink"
    );
    // (2) The visible box's mirror region IS green — the child overflows.
    assert!(
        is_green(pixmap, visible_spill.0, visible_spill.1),
        "overflow:visible clipped its child — expected green at {visible_spill:?}"
    );
    // (3) Inside the hidden box the child's green still shows (the clip keeps what
    // is inside the box, only removing the spill).
    assert!(
        is_green(pixmap, hidden_overlap.0, hidden_overlap.1),
        "the clipped child should still paint INSIDE the box at {hidden_overlap:?}"
    );
    println!(
        "[verify] hidden spill {hidden_spill:?} is white (clipped); visible spill {visible_spill:?} is green (overflows); in-box child {hidden_overlap:?} is green"
    );
    println!("[verify] OK — overflow:hidden clips the subtree; overflow:visible does not");
}

/// The workspace-relative output path `target/<name>`.
fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
