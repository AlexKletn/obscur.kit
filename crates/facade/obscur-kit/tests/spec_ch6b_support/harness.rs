//! Shared harness for the `spec_ch6b_*` **specification-derived** test files
//! (spec §6.8-§6.19, the platform-service sections).
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch6b_*.rs` files was written against the prose of obscur.kit §6. This
//! module exists only to give those cases a compiling home; it states no
//! expectation of its own and nothing here may be "fixed" to make a case pass.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// Compiles authored CSS text through the real CSS frontend.
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// An element node carrying `id`, with `kids` as its children.
#[must_use]
pub fn el(raw: u64, id: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(raw))
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// Renders one real frame of `v` under `css` in a fresh `w x h` runtime.
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
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
        assert!(i < kids.len(), "child index {i} out of range");
        node = kids[i];
    }
    node
}

/// The arranged origin of `node`, as `(x, y)`.
#[must_use]
pub fn origin_of(rt: &UiRuntime, node: NodeId) -> (f32, f32) {
    let b = rt.layout_boxes().get(&node).expect("an arranged box");
    (b.origin().x, b.origin().y)
}
