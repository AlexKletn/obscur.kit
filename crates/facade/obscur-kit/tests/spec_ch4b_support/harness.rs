//! Shared harness for the `spec_ch4b_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch4b_*.rs` files was derived from the obscur.kit specification's §4
//! clauses (the sections an earlier pass did not cover). This module exists only
//! to give those cases a compiling home; it holds no expectations of its own.
//!
//! Nothing here may be "fixed" to make a case pass. A failing `spec_ch4b_*` test
//! is a claim about the implementation, not about the test.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

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

/// Renders one real frame of `v` under a single authored stylesheet.
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = runtime(w, h);
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
    rt
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
            .children()
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

/// The node the runtime's real hit test resolves at window point `(x, y)`.
///
/// Drives a pointer `Move` through [`UiRuntime::handle_pointer`], which is the
/// runtime's only hit-testing entry point.
#[must_use]
pub fn hit(rt: &mut UiRuntime, x: f32, y: f32) -> Option<NodeId> {
    let event = PointerEvent::new(
        PointerId::from_raw(1),
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(x, y),
    );
    rt.handle_pointer(event).target
}

/// Every node's parent, computed by walking the retained tree's child lists.
#[must_use]
pub fn parent_of(rt: &UiRuntime, node: NodeId) -> Option<NodeId> {
    rt.tree()
        .iter()
        .find(|(_, n)| n.children().contains(&node))
        .map(|(id, _)| id)
}

/// The hit chain at `(x, y)`: the hit target followed by its tree ancestors,
/// topmost-first through the root. Empty when nothing was hit.
#[must_use]
pub fn hit_chain(rt: &mut UiRuntime, x: f32, y: f32) -> Vec<NodeId> {
    let Some(target) = hit(rt, x, y) else {
        return Vec::new();
    };
    let mut chain = vec![target];
    let mut cursor = target;
    while let Some(p) = parent_of(rt, cursor) {
        chain.push(p);
        cursor = p;
    }
    chain
}
