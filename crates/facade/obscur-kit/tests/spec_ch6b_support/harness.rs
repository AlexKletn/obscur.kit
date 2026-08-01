#![doc = include_str!("harness.doc/_module.md")]
#![allow(dead_code, unreachable_pub)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Insets, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{ID, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("harness.doc/compile.md")]
#[must_use]
pub fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(raw: u64, id: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(raw))
        .with_prop(ViewProperty::new(ID, ViewValue::Text(id.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/drive.md")]
#[must_use]
pub fn drive(w: f32, h: f32, css: &str, v: &View) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
    rt
}

#[doc = include_str!("harness.doc/drive_with_insets.md")]
#[must_use]
pub fn drive_with_insets(w: f32, h: f32, css: &str, v: &View, insets: Insets) -> UiRuntime {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
    rt.set_safe_area_insets(insets);
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
    rt
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
        assert!(i < kids.len(), "child index {i} out of range");
        node = kids[i];
    }
    node
}

#[doc = include_str!("harness.doc/origin_of.md")]
#[must_use]
pub fn origin_of(rt: &UiRuntime, node: NodeId) -> (f32, f32) {
    let b = rt.layout_boxes().get(&node).expect("an arranged box");
    (b.origin().x, b.origin().y)
}
