//! Shared harness for the `spec_ch3_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch3_*.rs` files was written by an agent reading obscur.kit §3 ("State,
//! transactions & runtime") **without opening any source in this repository**.
//! This module exists only to give those cases a compiling home; it contains no
//! expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch3_*` test
//! is a claim about the implementation, not about the test.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{AffineTransform, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// One deterministic vsync tick, in seconds (the 16 ms step the §3.5 cases use).
pub const VSYNC: f32 = 0.016;

/// One deterministic vsync tick, in nanoseconds.
pub const VSYNC_NANOS: u64 = 16_000_000;

/// Compiles authored CSS text into a matchable stylesheet through the real CSS
/// frontend (`source -> parse -> compile`).
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// A text node.
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// A runtime with a `w x h` viewport.
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

/// Walks from the tree root down `path` (child indices) to a [`NodeId`].
#[must_use]
pub fn at(rt: &UiRuntime, path: &[usize]) -> NodeId {
    let mut node = rt.tree().root().expect("a root after a frame");
    for &i in path {
        let kids = rt
            .tree()
            .get(node)
            .expect("a live node")
            .child_nodes()
            .to_vec();
        assert!(
            i < kids.len(),
            "child index {i} out of range: node has {} children",
            kids.len()
        );
        node = kids[i];
    }
    node
}

/// The absolute border-box `(x, y, w, h)` of a node.
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

/// Asserts `actual ≈ expected` within a hundredth of a pixel.
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

/// A node's computed `color`.
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

/// The text content of the first text node in the retained tree.
#[must_use]
pub fn tree_text(rt: &UiRuntime) -> Option<String> {
    rt.tree()
        .iter()
        .find_map(|(_, node)| node.kind().text().map(str::to_owned))
}

/// Every `PushTransform` affine in the last frame's display list, in paint order.
#[must_use]
pub fn transforms(rt: &UiRuntime) -> Vec<AffineTransform> {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::PushTransform(t) => Some(*t),
            _ => None,
        })
        .collect()
}

/// A stable, comparable rendering of the whole presented frame: the display list
/// plus the arranged geometry. Two runs that agree here agree on paint.
#[must_use]
pub fn frame_fingerprint(rt: &UiRuntime) -> String {
    let mut boxes: Vec<String> = rt
        .layout_boxes()
        .iter()
        .map(|(node, b)| {
            format!(
                "{}:{:?},{:?}",
                node.raw(),
                (b.origin().x, b.origin().y),
                (b.size().width, b.size().height)
            )
        })
        .collect();
    boxes.sort();
    format!(
        "commands={:?}|boxes={:?}",
        rt.snapshot().display_list().commands(),
        boxes
    )
}

/// Drains every due frame over a fixed `desired` view, returning how many frames
/// the scheduler actually presented. A runtime that owes nothing presents zero.
pub fn drain_frames(rt: &mut UiRuntime, desired: &View, sheets: &[Stylesheet]) -> usize {
    let mut presented = 0usize;
    // A bound so a runtime that never retires its request fails the assertion
    // rather than hanging the suite.
    while presented < 10_000 {
        if rt.run_due_frame(desired, sheets).is_none() {
            break;
        }
        presented += 1;
    }
    presented
}
