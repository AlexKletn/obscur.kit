#![doc = include_str!("harness.doc/_module.md")]
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

#[doc = include_str!("harness.doc/VSYNC.md")]
pub const VSYNC: f32 = 0.016;

#[doc = include_str!("harness.doc/VSYNC_NANOS.md")]
pub const VSYNC_NANOS: u64 = 16_000_000;

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/txt.md")]
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

#[doc = include_str!("harness.doc/at.md")]
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

#[doc = include_str!("harness.doc/geom.md")]
#[must_use]
pub fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("the node was arranged");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

#[doc = include_str!("harness.doc/approx.md")]
pub fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-2,
        "{what}: expected {expected}, got {actual}"
    );
}

#[doc = include_str!("harness.doc/color_of.md")]
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

#[doc = include_str!("harness.doc/tree_text.md")]
#[must_use]
pub fn tree_text(rt: &UiRuntime) -> Option<String> {
    rt.tree()
        .iter()
        .find_map(|(_, node)| node.kind().text().map(str::to_owned))
}

#[doc = include_str!("harness.doc/transforms.md")]
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

#[doc = include_str!("harness.doc/frame_fingerprint.md")]
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

#[doc = include_str!("harness.doc/drain_frames.md")]
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
