#![doc = include_str!("margin_collapse_proof.doc/_module.md")]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use obscur_geometry::BoxConstraints;
use obscur_kit::geometry::{Point, Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId};
use obscur_kit::layout::algorithm::{FixedChildMeasurer, FlowLayout, LayoutAlgorithm, LayoutChild};
use obscur_kit::layout::model::{LayoutBox, LayoutStyle, MarginEdges};
use obscur_kit::render::frame::{FrameSubmission, RenderFrame};
use obscur_kit::render::target::{BitmapTarget, RenderTarget};
use obscur_kit::render::{Renderer, RendererId};
use obscur_kit::render_cpu::renderer::CpuRenderer;
use obscur_kit::runtime::orchestration::{ComputedStyles, LayoutBoxes, SceneCoordinator};
use obscur_kit::scene::command::DisplayList;
use obscur_kit::scene::scene::{SceneGeneration, SceneSnapshot};
use obscur_kit::style::computed::{ComputedStyle, ComputedValue};
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::tree::node::{LogicalParent, NodeKind};
use obscur_kit::tree::tree::RetainedTree;

const VIEWPORT: Size = Size::new(560.0, 220.0);
const CONTAINER_W: f32 = 90.0;
const COL_Y: f32 = 30.0;
const COLS_X: [f32; 4] = [20.0, 160.0, 300.0, 440.0];

#[doc = include_str!("margin_collapse_proof.doc/child.md")]
fn child(raw: u64, top: f32, bottom: f32) -> LayoutChild {
    let style = LayoutStyle {
        margin: MarginEdges::px(top, 0.0, bottom, 0.0),
        ..LayoutStyle::default()
    };
    LayoutChild::new(NodeId::from_raw(raw), style)
}

#[doc = include_str!("margin_collapse_proof.doc/Case.md")]
struct Case {
    label: &'static str,
    container: LayoutStyle,
    children: Vec<LayoutChild>,
    sizes: Vec<Size>,
    #[doc = include_str!("margin_collapse_proof.doc/b_index.md")]
    b_index: usize,
    expected_b_top: f32,
}

fn cases() -> Vec<Case> {
    let gap0 = LayoutStyle::default();
    let gap10 = LayoutStyle {
        gap: 10.0,
        ..LayoutStyle::default()
    };
    vec![
        // (a) collapse to max: 20 vs 30 → 30.
        Case {
            label: "(a) mb20 / mt30 → 30 (max)",
            container: gap0,
            children: vec![child(1, 0.0, 20.0), child(2, 30.0, 0.0)],
            sizes: vec![Size::new(CONTAINER_W, 40.0), Size::new(CONTAINER_W, 50.0)],
            b_index: 1,
            expected_b_top: 70.0,
        },
        // (b) mixed sum: 20 + (−8) → 12.
        Case {
            label: "(b) mb20 / mt-8 → 12 (sum)",
            container: gap0,
            children: vec![child(1, 0.0, 20.0), child(2, -8.0, 0.0)],
            sizes: vec![Size::new(CONTAINER_W, 40.0), Size::new(CONTAINER_W, 50.0)],
            b_index: 1,
            expected_b_top: 52.0,
        },
        // (c) empty spacer collapses through: one 15px gap.
        Case {
            label: "(c) empty m15 spacer → 15 (one gap)",
            container: gap0,
            children: vec![child(1, 0.0, 0.0), child(2, 15.0, 15.0), child(3, 0.0, 0.0)],
            sizes: vec![
                Size::new(CONTAINER_W, 40.0),
                Size::new(CONTAINER_W, 0.0),
                Size::new(CONTAINER_W, 50.0),
            ],
            b_index: 2,
            expected_b_top: 55.0,
        },
        // (d) control: no margins, gap 10 → 10 (unchanged).
        Case {
            label: "(d) margins 0 / gap 10 → 10 (control)",
            container: gap10,
            children: vec![child(1, 0.0, 0.0), child(2, 0.0, 0.0)],
            sizes: vec![Size::new(CONTAINER_W, 40.0), Size::new(CONTAINER_W, 50.0)],
            b_index: 1,
            expected_b_top: 50.0,
        },
    ]
}

fn main() {
    println!("== obscur-kit vertical margin collapsing proof (CSS 2.1 §8.3.1, headless CPU) ==\n");

    let white = StyleColor::rgba(1.0, 1.0, 1.0, 1.0);
    let grey = StyleColor::rgba(0.80, 0.80, 0.82, 1.0); // container: shows the gaps
    let blue = StyleColor::rgba(0.20, 0.40, 0.85, 1.0); // block A
    let green = StyleColor::rgba(0.20, 0.75, 0.35, 1.0); // block B

    let mut tree = RetainedTree::new();
    let root = tree.alloc(NodeKind::Root);
    tree.set_root(root);

    let mut computed: ComputedStyles = ComputedStyles::default();
    computed.insert(root, background(white));
    let mut boxes: LayoutBoxes = LayoutBoxes::default();
    boxes.insert(
        root,
        LayoutBox::new(Rect::new(Point::new(0.0, 0.0), VIEWPORT)),
    );

    let mut raw = 10u64;
    // For each case: run the REAL FlowLayout, self-assert the y-positions, then
    // wire the computed boxes into the tree for rendering.
    let mut b_edges = Vec::new();
    for (col, case) in cases().into_iter().enumerate() {
        let mut measurer = FixedChildMeasurer::new(case.sizes.clone());
        let out = FlowLayout.layout(
            &case.container,
            &case.children,
            BoxConstraints::loose(Size::new(CONTAINER_W, 1000.0)),
            &mut measurer,
        );
        let child_boxes = out.child_boxes();

        // Load-bearing self-assert: FlowLayout placed B at the collapsed edge.
        let b_top = child_boxes[case.b_index].origin().y;
        assert!(
            (b_top - case.expected_b_top).abs() < 1e-4,
            "{}: B top {b_top} != expected {}",
            case.label,
            case.expected_b_top
        );
        // A's bottom edge (block A is always child 0 here).
        let a_bottom = child_boxes[0].origin().y + child_boxes[0].size().height;
        println!(
            "[layout] {:<38} A.bottom={a_bottom:>5}  B.top={b_top:>5}  Δ={:>5}  container.h={}",
            case.label,
            b_top - a_bottom,
            out.size().height,
        );

        // Container node at this column, filled grey, sized to FlowLayout output.
        let origin = Point::new(COLS_X[col], COL_Y);
        let container = element(&mut tree, root, raw);
        raw += 1;
        computed.insert(container, background(grey));
        boxes.insert(container, LayoutBox::new(Rect::new(origin, out.size())));

        // Children absolute = container origin + FlowLayout child box; blue A,
        // green B, spacer (zero height) skipped.
        for (i, cb) in child_boxes.iter().enumerate() {
            if cb.size().height == 0.0 {
                continue;
            }
            let node = element(&mut tree, container, raw);
            raw += 1;
            let color = if i == 0 { blue } else { green };
            computed.insert(node, background(color));
            let abs = Point::new(origin.x + cb.origin().x, origin.y + cb.origin().y);
            boxes.insert(node, LayoutBox::new(Rect::new(abs, cb.size())));
        }

        b_edges.push((COLS_X[col], COL_Y, a_bottom, b_top));
    }

    // Drive the REAL coordinator + software renderer.
    let list = SceneCoordinator::new().compose(&tree, &computed, &boxes, None);
    let pixmap = render(list);

    // Machine-check the render: in every column the collapsed GAP (between A's
    // bottom and B's top) is grey container, and B's top edge is green.
    for (label, (cx, cy, a_bottom, b_top)) in cases().iter().map(|c| c.label).zip(b_edges) {
        let px = (cx + CONTAINER_W / 2.0) as u32;
        let gap_mid = (cy + (a_bottom + b_top) / 2.0) as u32;
        let b_probe = (cy + b_top + 4.0) as u32;
        assert!(
            is_grey(&pixmap, px, gap_mid),
            "{label}: collapsed gap at ({px},{gap_mid}) should be grey container"
        );
        assert!(
            is_green(&pixmap, px, b_probe),
            "{label}: block B at ({px},{b_probe}) should be green"
        );
    }
    println!("\n[verify] OK — every column's collapsed gap is grey and B renders green below it");

    let png = pixmap.encode_png().expect("encode png");
    let out = output_path("margin_collapse_proof.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create target dir");
    }
    std::fs::write(&out, &png).expect("write png");
    println!(
        "[png] wrote {} × {} → {} ({} bytes)",
        pixmap.width(),
        pixmap.height(),
        out.display(),
        png.len(),
    );
}

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

fn background(color: StyleColor) -> ComputedStyle {
    let mut style = ComputedStyle::new();
    style.set(PropertyId::BackgroundColor, ComputedValue::color(color));
    style
}

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

fn is_green(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> bool {
    pixmap
        .pixel(x, y)
        .is_some_and(|p| p.green() > 140 && p.red() < 130 && p.blue() < 150)
}

fn is_grey(pixmap: &tiny_skia::Pixmap, x: u32, y: u32) -> bool {
    pixmap.pixel(x, y).is_some_and(|p| {
        let (r, g, b) = (p.red(), p.green(), p.blue());
        r > 180 && r < 225 && g > 180 && g < 225 && b > 180 && b < 230
    })
}

fn output_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("target")
        .join(name)
}
